use crate::{
    actors::{
        encrypts::{CreateWithEncryption, EncryptContent, VerifyEncryption, WriteWithEncryption},
        recovery::{LocalData, OffsetCounter},
        state::{MatchUpdate, PreviousRegistry, State},
        uniques::{CreateWithUniqueKeys, WriteWithUniqueKeys},
        wql::{DeleteId, InsertEntityContent, UpdateContentEntityContent, UpdateSetEntityContent},
    },
    core::wql::update_content_state,
    model::{
        wql::{InsertArgs, MatchUpdateArgs, UpdateArgs},
        DataAtomicUsize, DataEncryptContext, DataExecutor, DataLocalContext, DataU32,
        DataUniquenessContext,
    },
};
use crate::{
    actors::{
        uniques::CheckForUniqueKeys,
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
    local_data: DataLocalContext,
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
            create_controller(entity, local_data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::Delete(entity, uuid)) => {
            delete_controller(entity, uuid, local_data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::Insert(entity, content)) => {
            insert_controller(
                InsertArgs::new(entity, content),
                local_data.into_inner(),
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
                local_data.into_inner(),
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
                local_data.into_inner(),
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
                local_data.into_inner(),
                bytes_counter,
                uniqueness,
                encryption,
                hashing_cost,
                actor,
            )
            .await
        }
        Ok(Wql::Evict(entity, uuid)) => {
            evict_controller(entity, uuid, local_data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::CheckValue(entity, uuid, content)) => {
            check_value_controller(entity, uuid, content, local_data, encryption, actor).await
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
    local_data: DataLocalContext,
    encryption: DataEncryptContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    if let Ok(guard) = encryption.lock() {
        if guard.contains_key(&entity) {
            let encrypts = guard.get(&entity).unwrap();
            let non_encrypt_keys = content
                .iter()
                .filter(|(k, _)| !encrypts.contains(&(*k).to_string()))
                .map(|(_, v)| v.to_owned())
                .collect::<Vec<String>>();

            if !non_encrypt_keys.is_empty() {
                return Err(Error::CheckNonEncryptedKeys(non_encrypt_keys));
            }
        }
    };

    let local_data = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if !local_data.contains_key(&entity) {
            return Err(Error::EntityNotCreated(entity));
        }
        local_data.clone()
    };

    let previous_entry = local_data.get(&entity).unwrap().get(&uuid).unwrap();
    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let state = actor.send(State(previous_state_str)).await??;
    let keys = content
        .keys()
        .map(ToOwned::to_owned)
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
    local_data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    actor: DataExecutor,
) -> Result<String, Error> {
    let local_data = {
        let mut local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if local_data.contains_key(&entity) {
            return Err(Error::EntityAlreadyCreated(entity));
        }

        local_data.insert(entity.clone(), BTreeMap::new());
        local_data.clone()
    };
    actor.send(LocalData::new(local_data)).await??;

    let message = format!("Entity `{}` created", &entity);
    let (offset, is_empty) = actor.send(CreateEntity::new(&entity)).await??;

    if is_empty {
        bytes_counter.store(0, Ordering::SeqCst);
    }
    bytes_counter.fetch_add(offset, Ordering::SeqCst);
    actor
        .send(OffsetCounter::new(bytes_counter.load(Ordering::SeqCst)))
        .await??;

    Ok(CreateEntityResponse::new(entity, message).write())
}

pub async fn evict_controller(
    entity: String,
    uuid: Option<Uuid>,
    local_data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    actor: DataExecutor,
) -> Result<String, Error> {
    if uuid.is_none() {
        let message = format!("Entity {} evicted", &entity);
        let (offset, is_empty) = actor.send(EvictEntity::new(&entity)).await??;

        if is_empty {
            bytes_counter.store(0, Ordering::SeqCst);
        }
        bytes_counter.fetch_add(offset, Ordering::SeqCst);
        actor
            .send(OffsetCounter::new(bytes_counter.load(Ordering::SeqCst)))
            .await??;

        let local_data = {
            let mut local_data = if let Ok(guard) = local_data.lock() {
                guard
            } else {
                return Err(Error::LockData);
            };
            local_data.remove(&entity);
            local_data.clone()
        };

        actor.send(LocalData::new(local_data)).await??;
        Ok(DeleteOrEvictEntityResponse::new(entity, None, message).write())
    } else {
        let id = uuid.unwrap();
        let (offset, is_empty) = actor.send(EvictEntityId::new(&entity, id)).await??;

        if is_empty {
            bytes_counter.store(0, Ordering::SeqCst);
        }

        bytes_counter.fetch_add(offset, Ordering::SeqCst);
        actor
            .send(OffsetCounter::new(bytes_counter.load(Ordering::SeqCst)))
            .await??;

        let local_data = {
            let mut local_data = if let Ok(guard) = local_data.lock() {
                guard
            } else {
                return Err(Error::LockData);
            };
            if let Some(d) = local_data.get_mut(&entity) {
                d.remove(&id);
            }
            local_data.clone()
        };
        actor.send(LocalData::new(local_data)).await??;

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
        let uniqueness_data = uniqueness.into_inner();
        actor
            .send(WriteWithUniqueKeys {
                entity: entity.to_owned(),
                uniques: uniques.clone(),
            })
            .await??;
        actor
            .send(CreateWithUniqueKeys {
                entity: entity.to_owned(),
                uniques,
                data: uniqueness_data,
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
        let local_data = encryption.into_inner();
        actor
            .send(WriteWithEncryption {
                entity: entity.to_owned(),
                encrypts: encrypts.clone(),
            })
            .await??;
        actor
            .send(CreateWithEncryption {
                entity: entity.to_owned(),
                encrypts,
                data: local_data,
            })
            .await??;
        Ok(())
    }
}

pub async fn insert_controller(
    args: InsertArgs,
    local_data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    uniqueness: DataUniquenessContext,
    encryption: DataEncryptContext,
    hashing_cost: DataU32,
    actor: DataExecutor,
) -> Result<String, Error> {
    let mut offset = bytes_counter.load(Ordering::SeqCst);
    let encrypted_content = actor
        .send(EncryptContent::new(
            &args.entity,
            args.content,
            encryption.into_inner(),
            *hashing_cost.into_inner(),
        ))
        .await??;
    let content_log =
        to_string_pretty(&encrypted_content, pretty_config()).map_err(Error::Serialization)?;

    {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if !local_data.contains_key(&args.entity) {
            return Err(Error::EntityNotCreated(args.entity));
        }
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUniqueKeys {
            entity: args.entity.to_owned(),
            content: encrypted_content,
            uniqueness,
        })
        .await??;

    let content_value = actor
        .send(InsertEntityContent::new(&args.entity, &content_log))
        .await??;

    if content_value.3 {
        bytes_counter.store(0, Ordering::SeqCst);
        offset = 0;
    }

    let local_data_register = DataRegister {
        offset,
        bytes_length: content_value.2,
        file_name: content_value.0.format("data/%Y_%m_%d.log").to_string(),
    };

    let local_data = {
        let mut local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if let Some(map) = local_data.get_mut(&args.entity) {
            map.insert(content_value.1, local_data_register);
        }
        local_data.clone()
    };

    actor.send(LocalData::new(local_data)).await??;

    bytes_counter.fetch_add(content_value.2, Ordering::SeqCst);
    actor
        .send(OffsetCounter::new(bytes_counter.load(Ordering::SeqCst)))
        .await??;

    let message = format!(
        "Entity {} inserted with Uuid {}",
        &args.entity, &content_value.1
    );
    Ok(InsertEntityResponse::new(args.entity, content_value.1, message).write())
}

pub async fn update_set_controller(
    args: UpdateArgs,
    local_data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    uniqueness: DataUniquenessContext,
    encryption: DataEncryptContext,
    hashing_cost: DataU32,
    actor: DataExecutor,
) -> Result<String, Error> {
    let mut offset = bytes_counter.load(Ordering::SeqCst);
    let encrypted_content = actor
        .send(EncryptContent::new(
            &args.entity,
            args.content,
            encryption.into_inner(),
            *hashing_cost.into_inner(),
        ))
        .await??;
    let content_log =
        to_string_pretty(&encrypted_content, pretty_config()).map_err(Error::Serialization)?;

    {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if !local_data.contains_key(&args.entity) {
            return Err(Error::EntityNotCreated(args.entity));
        } else if local_data.contains_key(&args.entity)
            && !local_data.get(&args.entity).unwrap().contains_key(&args.id)
        {
            return Err(Error::UuidNotCreatedForEntity(args.entity, args.id));
        }
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUniqueKeys {
            entity: args.entity.to_owned(),
            content: encrypted_content.to_owned(),
            uniqueness,
        })
        .await??;

    let previous_entry = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let previous_entry = local_data.get(&args.entity).unwrap().get(&args.id).unwrap();
        previous_entry.clone()
    };

    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let mut previous_state = actor.send(State(previous_state_str)).await??;

    encrypted_content.into_iter().for_each(|(k, v)| {
        let local_state = previous_state.entry(k).or_insert_with(|| v.clone());
        *local_state = v;
    });

    let state_log =
        to_string_pretty(&previous_state, pretty_config()).map_err(Error::Serialization)?;

    let content_value = actor
        .send(UpdateSetEntityContent::new(
            &args.entity,
            &state_log,
            &content_log,
            args.id,
            &to_string_pretty(&previous_entry.clone(), pretty_config())
                .map_err(Error::Serialization)?,
        ))
        .await??;

    if content_value.2 {
        bytes_counter.store(0, Ordering::SeqCst);
        offset = 0;
    }

    let local_data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("data/%Y_%m_%d.log").to_string(),
    };

    let local_data = {
        let mut local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if let Some(map) = local_data.get_mut(&args.entity) {
            if let Some(reg) = map.get_mut(&args.id) {
                *reg = local_data_register;
            }
        }
        local_data.clone()
    };
    actor.send(LocalData::new(local_data)).await??;

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);
    actor
        .send(OffsetCounter::new(bytes_counter.load(Ordering::SeqCst)))
        .await??;
    let message = format!("Entity {} with Uuid {} updated", &args.entity, &args.id);
    Ok(UpdateEntityResponse::new(args.entity, args.id, state_log, message).write())
}

pub async fn update_content_controller(
    args: UpdateArgs,
    local_data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    uniqueness: DataUniquenessContext,
    encryption: DataEncryptContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let mut offset = bytes_counter.load(Ordering::SeqCst);
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
        to_string_pretty(&args.content, pretty_config()).map_err(Error::Serialization)?;

    {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if !local_data.contains_key(&args.entity) {
            return Err(Error::EntityNotCreated(args.entity));
        } else if local_data.contains_key(&args.entity)
            && !local_data.get(&args.entity).unwrap().contains_key(&args.id)
        {
            return Err(Error::UuidNotCreatedForEntity(args.entity, args.id));
        }
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUniqueKeys {
            entity: args.entity.to_owned(),
            content: args.content.to_owned(),
            uniqueness,
        })
        .await??;

    let previous_entry = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let previous_entry = local_data.get(&args.entity).unwrap().get(&args.id).unwrap();
        previous_entry.clone()
    };

    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let mut previous_state = actor.send(State(previous_state_str)).await??;

    args.content
        .into_iter()
        .for_each(|(k, v)| update_content_state(&mut previous_state, k, v));

    let state_log =
        to_string_pretty(&previous_state, pretty_config()).map_err(Error::Serialization)?;

    let content_value = actor
        .send(UpdateContentEntityContent::new(
            &args.entity,
            &state_log,
            &content_log,
            args.id,
            &to_string_pretty(&previous_entry, pretty_config()).map_err(Error::Serialization)?,
        ))
        .await??;

    if content_value.2 {
        bytes_counter.store(0, Ordering::SeqCst);
        offset = 0;
    }
    let local_data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("data/%Y_%m_%d.log").to_string(),
    };
    let local_data = {
        let mut local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if let Some(map) = local_data.get_mut(&args.entity) {
            if let Some(reg) = map.get_mut(&args.id) {
                *reg = local_data_register;
            }
        }
        local_data.clone()
    };
    actor.send(LocalData::new(local_data)).await??;

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);
    actor
        .send(OffsetCounter::new(bytes_counter.load(Ordering::SeqCst)))
        .await??;

    let message = format!("Entity {} with Uuid {} updated", &args.entity, &args.id);
    Ok(UpdateEntityResponse::new(args.entity, args.id, state_log, message).write())
}

pub async fn delete_controller(
    entity: String,
    id: String,
    local_data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    actor: DataExecutor,
) -> Result<String, Error> {
    let uuid = Uuid::from_str(&id)?;
    let message = format!("Entity {} with Uuid {} deleted", &entity, id);
    let mut offset = bytes_counter.load(Ordering::SeqCst);

    let previous_entry = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if !local_data.contains_key(&entity) {
            return Err(Error::EntityNotCreated(entity));
        } else if local_data.contains_key(&entity)
            && !local_data.get(&entity).unwrap().contains_key(&uuid)
        {
            return Err(Error::UuidNotCreatedForEntity(entity, uuid));
        }

        let previous_entry = local_data.get(&entity).unwrap().get(&uuid).unwrap();
        previous_entry.clone()
    };

    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let two_registries_ago = actor.send(PreviousRegistry(previous_state_str)).await??;

    let state_to_be = if let Some(reg) = two_registries_ago {
        let state_str = actor.send(reg.to_owned()).await??;
        (actor.send(State(state_str)).await??, reg.to_owned())
    } else {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let insert_reg = local_data.get(&entity).unwrap().get(&uuid).unwrap();
        (HashMap::new(), insert_reg.to_owned())
    };

    let content_log =
        to_string_pretty(&state_to_be.0, pretty_config()).map_err(Error::Serialization)?;

    let previous_register_log =
        to_string_pretty(&state_to_be.1, pretty_config()).map_err(Error::Serialization)?;

    let content_value = actor
        .send(DeleteId::new(
            &entity,
            &content_log,
            uuid,
            &previous_register_log,
        ))
        .await??;

    if content_value.2 {
        bytes_counter.store(0, Ordering::SeqCst);
        offset = 0;
    }
    let local_data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("data/%Y_%m_%d.log").to_string(),
    };

    let local_data = {
        let mut local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if let Some(map) = local_data.get_mut(&entity) {
            if let Some(reg) = map.get_mut(&uuid) {
                *reg = local_data_register;
            }
        }
        local_data.clone()
    };

    actor.send(LocalData::new(local_data)).await??;

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);
    actor
        .send(OffsetCounter::new(bytes_counter.load(Ordering::SeqCst)))
        .await??;

    Ok(DeleteOrEvictEntityResponse::new(entity, Some(uuid), message).write())
}

pub async fn match_update_set_controller(
    args: MatchUpdateArgs,
    local_data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: DataAtomicUsize,
    uniqueness: DataUniquenessContext,
    encryption: DataEncryptContext,
    hashing_cost: DataU32,
    actor: DataExecutor,
) -> Result<String, Error> {
    let previous_entry = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if !local_data.contains_key(&args.entity) {
            return Err(Error::EntityNotCreated(args.entity));
        } else if local_data.contains_key(&args.entity)
            && !local_data.get(&args.entity).unwrap().contains_key(&args.id)
        {
            return Err(Error::UuidNotCreatedForEntity(args.entity, args.id));
        }

        let previous_entry = local_data.get(&args.entity).unwrap().get(&args.id).unwrap();
        previous_entry.clone()
    };
    let previous_state_str = actor.send(previous_entry.to_owned()).await??;
    let mut previous_state = actor.send(State(previous_state_str)).await??;

    actor
        .send(MatchUpdate {
            conditions: args.conditions,
            previous_state: previous_state.clone(),
        })
        .await??;

    let mut offset = bytes_counter.load(Ordering::SeqCst);

    let encrypted_content = actor
        .send(EncryptContent::new(
            &args.entity,
            args.content.clone(),
            encryption.into_inner(),
            *hashing_cost.into_inner(),
        ))
        .await??;
    let content_log =
        to_string_pretty(&encrypted_content, pretty_config()).map_err(Error::Serialization)?;

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUniqueKeys {
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
        to_string_pretty(&previous_state, pretty_config()).map_err(Error::Serialization)?;

    let content_value = actor
        .send(UpdateSetEntityContent {
            name: args.entity.clone(),
            current_state: state_log.clone(),
            content_log,
            id: args.id,
            previous_registry: to_string_pretty(&previous_entry, pretty_config())
                .map_err(Error::Serialization)?,
        })
        .await??;

    if content_value.2 {
        bytes_counter.store(0, Ordering::SeqCst);
        offset = 0;
    }
    let local_data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("data/%Y_%m_%d.log").to_string(),
    };

    let local_data = {
        let mut local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        if let Some(map) = local_data.get_mut(&args.entity) {
            if let Some(reg) = map.get_mut(&args.id) {
                *reg = local_data_register;
            }
        }
        local_data.clone()
    };

    actor.send(LocalData::new(local_data)).await??;

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);
    actor
        .send(OffsetCounter::new(bytes_counter.load(Ordering::SeqCst)))
        .await??;

    let message = format!("Entity {} with Uuid {} updated", &args.entity, &args.id);
    Ok(UpdateEntityResponse::new(args.entity, args.id, state_log, message).write())
}
