use crate::{
    actors::{
        encrypts::{CreateEncrypts, EncryptContent, VerifyEncryption, WriteEncrypts},
        state::{MatchUpdate, PreviousRegistry, State},
        uniques::{CreateUniques, WriteUniques},
        wql::{DeleteId, InsertEntityContent, UpdateContentEntityContent, UpdateSetEntityContent},
    },
    model::{
        wql::{InsertArgs, MatchUpdateArgs, UpdateArgs},
        DataAtomicUsize, DataEncryptContext, DataExecutor, DataLocalContext, DataU32,
        DataUniquenessContext,
    },
};
use crate::{
    actors::{
        uniques::CheckForUnique,
        wql::{CreateEntity, EvictEntity, EvictEntityId},
    },
    schemas::tx::CreateEntityResponse,
};
use crate::{
    model::{error::Error, DataRegister},
    schemas::tx::InsertEntityResponse,
};
use crate::{
    repository::local::LocalContext,
    schemas::tx::{DeleteOrEvictEntityResponse, UpdateEntityResponse},
};

use actix_web::{HttpResponse, Responder};
use ron::ser::{to_string_pretty, PrettyConfig};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    str::FromStr,
    sync::{atomic::Ordering, Arc, Mutex},
};
use uuid::Uuid;
use wql::{Types, Wql};

fn pretty_config() -> PrettyConfig {
    PrettyConfig::new()
        .with_indentor("".to_string())
        .with_new_line("".to_string())
}

pub async fn wql_handler(
    body: String,
    data: DataLocalContext,
    uniqueness: DataUniquenessContext,
    encryption: DataEncryptContext,
    bytes_counter: DataAtomicUsize,
    hashing_cost: DataU32,
    actor: DataExecutor,
) -> impl Responder {
    let query = wql::Wql::from_str(&body);
    let response = match query {
        Ok(Wql::CreateEntity(entity, uniques, encrypts)) => {
            let _ = create_unique_controller(&entity, uniques, uniqueness, &actor).await;
            let _ = create_encrypts_controller(&entity, encrypts, encryption, &actor).await;
            create_controller(entity, data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::Delete(entity, uuid)) => {
            delete_controller(entity, uuid, data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::Insert(entity, content)) => {
            insert_controller(
                InsertArgs::new(entity, content),
                data.into_inner(),
                bytes_counter,
                uniqueness,
                encryption,
                hashing_cost,
                actor,
            )
            .await
        }
        Ok(Wql::UpdateContent(entity, content, uuid)) => {
            update_content_controller(
                UpdateArgs::new(entity, content, uuid),
                data.into_inner(),
                bytes_counter,
                uniqueness,
                encryption,
                actor,
            )
            .await
        }
        Ok(Wql::UpdateSet(entity, content, uuid)) => {
            update_set_controller(
                UpdateArgs::new(entity, content, uuid),
                data.into_inner(),
                bytes_counter,
                uniqueness,
                encryption,
                hashing_cost,
                actor,
            )
            .await
        }
        Ok(Wql::MatchUpdate(entity, content, uuid, conditions)) => {
            match_update_set_controller(
                MatchUpdateArgs::new(entity, content, uuid, conditions),
                data.into_inner(),
                bytes_counter,
                uniqueness,
                encryption,
                hashing_cost,
                actor,
            )
            .await
        }
        Ok(Wql::Evict(entity, uuid)) => {
            evict_controller(entity, uuid, data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::CheckValue(entity, uuid, content)) => {
            check_value_controller(entity, uuid, content, data, encryption, actor).await
        }
        Ok(_) => Err(Error::SelectBadRequest),
        Err(e) => Err(Error::QueryFormat(e)),
    };

    match response {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}

pub async fn check_value_controller(
    entity: String,
    uuid: Uuid,
    content: HashMap<String, String>,
    data: DataLocalContext,
    encryption: DataEncryptContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    if let Ok(guard) = encryption.lock() {
        if guard.contains_key(&entity) {
            let encrypts = guard.get(&entity).unwrap();
            let non_encrypt_keys = content
                .iter()
                .filter(|(k, _)| !encrypts.contains(&k.to_string()))
                .map(|(_, v)| v.to_owned())
                .collect::<Vec<String>>();

            if !non_encrypt_keys.is_empty() {
                return Err(Error::CheckNonEncryptedKeys(non_encrypt_keys));
            }
        }
    };

    let data = if let Ok(guard) = data.lock() {
        guard
    } else {
        return Err(Error::LockData);
    };
    if !data.contains_key(&entity) {
        return Err(Error::EntityNotCreated(entity));
    }

    let previous_entry = data.get(&entity).unwrap().get(&uuid).unwrap();
    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let state = actor.send(State(previous_state_str)).await??;
    let keys = content
        .keys()
        .map(|k| k.to_owned())
        .collect::<HashSet<String>>();
    let filtered_state: HashMap<String, Types> = state
        .into_iter()
        .filter(|(k, _)| keys.contains(k))
        .collect();
    let results = actor
        .send(VerifyEncryption::new(filtered_state, content))
        .await??;
    Ok(results)
}

pub async fn create_controller(
    entity: String,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    actor: DataExecutor,
) -> Result<String, Error> {
    {
        let mut data = if let Ok(guard) = data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if !data.contains_key(&entity) {
            data.insert(entity.clone(), BTreeMap::new());
        } else {
            return Err(Error::EntityAlreadyCreated(entity));
        }
    }

    let message = format!("Entity `{}` created", &entity);
    let offset = actor.send(CreateEntity::new(&entity)).await??;

    bytes_counter.fetch_add(offset, Ordering::SeqCst);

    Ok(CreateEntityResponse::new(entity, message).write())
}

pub async fn evict_controller(
    entity: String,
    uuid: Option<Uuid>,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    actor: DataExecutor,
) -> Result<String, Error> {
    if uuid.is_none() {
        let message = format!("Entity {} evicted", &entity);
        let offset = actor.send(EvictEntity::new(&entity)).await??;
        bytes_counter.fetch_add(offset, Ordering::SeqCst);

        let mut data = if let Ok(guard) = data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        data.remove(&entity);
        Ok(DeleteOrEvictEntityResponse::new(entity, None, message).write())
    } else {
        let id = uuid.unwrap();
        let offset = actor.send(EvictEntityId::new(&entity, id)).await??;
        bytes_counter.fetch_add(offset, Ordering::SeqCst);

        let mut data = if let Ok(guard) = data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if let Some(d) = data.get_mut(&entity) {
            d.remove(&id);
        }

        let message = format!("Entity {} with id {} evicted", &entity, &id);
        Ok(DeleteOrEvictEntityResponse::new(entity, uuid, message).write())
    }
}

pub async fn create_unique_controller(
    entity: &str,
    uniques: Vec<String>,
    uniqueness: DataUniquenessContext,
    actor: &DataExecutor,
) -> Result<(), Error> {
    if uniques.is_empty() {
        Ok(())
    } else {
        let data = uniqueness.into_inner();
        actor
            .send(WriteUniques {
                entity: entity.to_owned(),
                uniques: uniques.clone(),
            })
            .await??;
        actor
            .send(CreateUniques {
                entity: entity.to_owned(),
                uniques,
                data,
            })
            .await??;
        Ok(())
    }
}

pub async fn create_encrypts_controller(
    entity: &str,
    encrypts: Vec<String>,
    encryption: DataEncryptContext,
    actor: &DataExecutor,
) -> Result<(), Error> {
    if encrypts.is_empty() {
        Ok(())
    } else {
        let data = encryption.into_inner();
        actor
            .send(WriteEncrypts {
                entity: entity.to_owned(),
                encrypts: encrypts.clone(),
            })
            .await??;
        actor
            .send(CreateEncrypts {
                entity: entity.to_owned(),
                encrypts,
                data,
            })
            .await??;
        Ok(())
    }
}

pub async fn insert_controller(
    args: InsertArgs,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    uniqueness: DataUniquenessContext,
    encryption: DataEncryptContext,
    hashing_cost: DataU32,
    actor: DataExecutor,
) -> Result<String, Error> {
    let offset = bytes_counter.load(Ordering::SeqCst);
    let encrypted_content = actor
        .send(EncryptContent::new(
            &args.entity,
            args.content,
            encryption.into_inner(),
            *hashing_cost.into_inner(),
        ))
        .await??;
    let content_log =
        to_string_pretty(&encrypted_content, pretty_config()).map_err(Error::SerializationError)?;

    let mut data = if let Ok(guard) = data.lock() {
        guard
    } else {
        return Err(Error::LockData);
    };
    if !data.contains_key(&args.entity) {
        return Err(Error::EntityNotCreated(args.entity));
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUnique {
            entity: args.entity.to_owned(),
            content: encrypted_content,
            uniqueness,
        })
        .await??;

    let content_value = actor
        .send(InsertEntityContent::new(&args.entity, &content_log))
        .await??;
    let data_register = DataRegister {
        offset,
        bytes_length: content_value.2,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&args.entity) {
        map.insert(content_value.1, data_register);
    }

    bytes_counter.fetch_add(content_value.2, Ordering::SeqCst);

    let message = format!(
        "Entity {} inserted with Uuid {}",
        &args.entity, &content_value.1
    );
    Ok(InsertEntityResponse::new(args.entity, content_value.1, message).write())
}

pub async fn update_set_controller(
    args: UpdateArgs,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    uniqueness: DataUniquenessContext,
    encryption: DataEncryptContext,
    hashing_cost: DataU32,
    actor: DataExecutor,
) -> Result<String, Error> {
    let offset = bytes_counter.load(Ordering::SeqCst);
    let encrypted_content = actor
        .send(EncryptContent::new(
            &args.entity,
            args.content,
            encryption.into_inner(),
            *hashing_cost.into_inner(),
        ))
        .await??;
    let content_log =
        to_string_pretty(&encrypted_content, pretty_config()).map_err(Error::SerializationError)?;

    let mut data = if let Ok(guard) = data.lock() {
        guard
    } else {
        return Err(Error::LockData);
    };
    if !data.contains_key(&args.entity) {
        return Err(Error::EntityNotCreated(args.entity));
    } else if data.contains_key(&args.entity)
        && !data.get(&args.entity).unwrap().contains_key(&args.id)
    {
        return Err(Error::UuidNotCreatedForEntity(args.entity, args.id));
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUnique {
            entity: args.entity.to_owned(),
            content: encrypted_content.to_owned(),
            uniqueness,
        })
        .await??;

    let previous_entry = data.get(&args.entity).unwrap().get(&args.id).unwrap();
    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let mut previous_state = actor.send(State(previous_state_str)).await??;

    encrypted_content.into_iter().for_each(|(k, v)| {
        let local_state = previous_state.entry(k).or_insert_with(|| v.clone());
        *local_state = v;
    });

    let state_log =
        to_string_pretty(&previous_state, pretty_config()).map_err(Error::SerializationError)?;

    let content_value = actor
        .send(UpdateSetEntityContent::new(
            &args.entity,
            &state_log,
            &content_log,
            args.id,
            &to_string_pretty(&previous_entry.clone(), pretty_config())
                .map_err(Error::SerializationError)?,
        ))
        .await??;

    let data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&args.entity) {
        if let Some(reg) = map.get_mut(&args.id) {
            *reg = data_register;
        }
    }

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);
    let message = format!("Entity {} with Uuid {} updated", &args.entity, &args.id);
    Ok(UpdateEntityResponse::new(args.entity, args.id, state_log, message).write())
}

pub async fn update_content_controller(
    args: UpdateArgs,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    uniqueness: DataUniquenessContext,
    encryption: DataEncryptContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let offset = bytes_counter.load(Ordering::SeqCst);
    if let Ok(guard) = encryption.lock() {
        if guard.contains_key(&args.entity) {
            let keys = args
                .content
                .iter()
                .filter(|(k, _)| guard.get(&args.entity).unwrap().contains(k.to_owned()))
                .map(|(k, _)| k.to_owned())
                .collect::<Vec<String>>();
            return Err(Error::UpdateContentEncryptKeys(keys));
        }
    } else {
        return Err(Error::LockData);
    };
    let content_log =
        to_string_pretty(&args.content, pretty_config()).map_err(Error::SerializationError)?;

    let mut data = if let Ok(guard) = data.lock() {
        guard
    } else {
        return Err(Error::LockData);
    };
    if !data.contains_key(&args.entity) {
        return Err(Error::EntityNotCreated(args.entity));
    } else if data.contains_key(&args.entity)
        && !data.get(&args.entity).unwrap().contains_key(&args.id)
    {
        return Err(Error::UuidNotCreatedForEntity(args.entity, args.id));
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUnique {
            entity: args.entity.to_owned(),
            content: args.content.to_owned(),
            uniqueness,
        })
        .await??;

    let previous_entry = data.get(&args.entity).unwrap().get(&args.id).unwrap();
    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let mut previous_state = actor.send(State(previous_state_str)).await??;

    args.content.into_iter().for_each(|(k, v)| {
        let local_state = previous_state
            .entry(k)
            .or_insert_with(|| v.default_values());
        match v {
            Types::Char(c) => {
                *local_state = Types::Char(c);
            }
            Types::Integer(i) => {
                if let Types::Integer(local) = *local_state {
                    *local_state = Types::Integer(local + i);
                }

                if let Types::Float(local) = *local_state {
                    *local_state = Types::Float(local + i as f64);
                }
            }
            Types::String(s) => {
                if let Types::String(local) = local_state {
                    *local_state = Types::String(local.to_owned() + &s);
                }
            }
            Types::Uuid(uuid) => {
                *local_state = Types::Uuid(uuid);
            }
            Types::Float(f) => {
                if let Types::Float(local) = *local_state {
                    *local_state = Types::Float(local + f);
                }

                if let Types::Integer(local) = *local_state {
                    *local_state = Types::Float(local as f64 + f);
                }
            }
            Types::Boolean(b) => {
                *local_state = Types::Boolean(b);
            }
            Types::Hash(_) => {}
            Types::Vector(mut v) => {
                if let Types::Vector(local) = local_state {
                    local.append(&mut v);
                    *local_state = Types::Vector(local.to_owned());
                }
            }
            Types::Map(m) => {
                if let Types::Map(local) = local_state {
                    m.iter().for_each(|(key, value)| {
                        local
                            .entry(key.to_owned())
                            .and_modify(|v| *v = value.to_owned())
                            .or_insert_with(|| value.to_owned());
                    });
                    *local_state = Types::Map(local.to_owned());
                }
            }
            Types::Nil => {
                *local_state = Types::Nil;
            }
            Types::Precise(p) => {
                *local_state = Types::Precise(p);
            }
        }
    });

    let state_log =
        to_string_pretty(&previous_state, pretty_config()).map_err(Error::SerializationError)?;

    let content_value = actor
        .send(UpdateContentEntityContent::new(
            &args.entity,
            &state_log,
            &content_log,
            args.id,
            &to_string_pretty(&previous_entry, pretty_config())
                .map_err(Error::SerializationError)?,
        ))
        .await??;

    let data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&args.entity) {
        if let Some(reg) = map.get_mut(&args.id) {
            *reg = data_register;
        }
    }

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);

    let message = format!("Entity {} with Uuid {} updated", &args.entity, &args.id);
    Ok(UpdateEntityResponse::new(args.entity, args.id, state_log, message).write())
}

pub async fn delete_controller(
    entity: String,
    id: String,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    actor: DataExecutor,
) -> Result<String, Error> {
    let uuid = Uuid::from_str(&id)?;
    let message = format!("Entity {} with Uuid {} deleted", &entity, id);
    let offset = bytes_counter.load(Ordering::SeqCst);

    let mut data = if let Ok(guard) = data.lock() {
        guard
    } else {
        return Err(Error::LockData);
    };
    if !data.contains_key(&entity) {
        return Err(Error::EntityNotCreated(entity));
    } else if data.contains_key(&entity) && !data.get(&entity).unwrap().contains_key(&uuid) {
        return Err(Error::UuidNotCreatedForEntity(entity, uuid));
    }

    let previous_entry = data.get(&entity).unwrap().get(&uuid).unwrap();
    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let two_registries_ago = actor.send(PreviousRegistry(previous_state_str)).await??;

    let state_to_be = match two_registries_ago {
        Some(reg) => {
            let state_str = actor.send(reg.to_owned()).await??;
            (actor.send(State(state_str)).await??, reg.to_owned())
        }
        None => {
            let insert_reg = data.get(&entity).unwrap().get(&uuid).unwrap();
            (HashMap::new(), insert_reg.to_owned())
        }
    };
    let content_log =
        to_string_pretty(&state_to_be.0, pretty_config()).map_err(Error::SerializationError)?;

    let previous_register_log =
        to_string_pretty(&state_to_be.1, pretty_config()).map_err(Error::SerializationError)?;

    let content_value = actor
        .send(DeleteId::new(
            &entity,
            &content_log,
            uuid,
            &previous_register_log,
        ))
        .await??;

    let data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&entity) {
        if let Some(reg) = map.get_mut(&uuid) {
            *reg = data_register;
        }
    }

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);

    Ok(DeleteOrEvictEntityResponse::new(entity, Some(uuid), message).write())
}

pub async fn match_update_set_controller(
    args: MatchUpdateArgs,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    uniqueness: DataUniquenessContext,
    encryption: DataEncryptContext,
    hashing_cost: DataU32,
    actor: DataExecutor,
) -> Result<String, Error> {
    let mut data = if let Ok(guard) = data.lock() {
        guard
    } else {
        return Err(Error::LockData);
    };
    if !data.contains_key(&args.entity) {
        return Err(Error::EntityNotCreated(args.entity));
    } else if data.contains_key(&args.entity)
        && !data.get(&args.entity).unwrap().contains_key(&args.id)
    {
        return Err(Error::UuidNotCreatedForEntity(args.entity, args.id));
    }

    let previous_entry = data.get(&args.entity).unwrap().get(&args.id).unwrap();
    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let mut previous_state = actor.send(State(previous_state_str)).await??;

    actor
        .send(MatchUpdate {
            conditions: args.conditions,
            previous_state: previous_state.clone(),
        })
        .await??;

    let offset = bytes_counter.load(Ordering::SeqCst);

    let encrypted_content = actor
        .send(EncryptContent::new(
            &args.entity,
            args.content.clone(),
            encryption.into_inner(),
            *hashing_cost.into_inner(),
        ))
        .await??;
    let content_log =
        to_string_pretty(&encrypted_content, pretty_config()).map_err(Error::SerializationError)?;

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUnique {
            entity: args.entity.to_owned(),
            content: args.content.to_owned(),
            uniqueness,
        })
        .await??;

    args.content.into_iter().for_each(|(k, v)| {
        let local_state = previous_state.entry(k).or_insert_with(|| v.clone());
        *local_state = v;
    });

    let state_log =
        to_string_pretty(&previous_state, pretty_config()).map_err(Error::SerializationError)?;

    let content_value = actor
        .send(UpdateSetEntityContent {
            name: args.entity.clone(),
            current_state: state_log.clone(),
            content_log,
            id: args.id,
            previous_registry: to_string_pretty(&previous_entry, pretty_config())
                .map_err(Error::SerializationError)?,
        })
        .await??;

    let data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&args.entity) {
        if let Some(reg) = map.get_mut(&args.id) {
            *reg = data_register;
        }
    }

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);

    let message = format!("Entity {} with Uuid {} updated", &args.entity, &args.id);
    Ok(UpdateEntityResponse::new(args.entity, args.id, state_log, message).write())
}
