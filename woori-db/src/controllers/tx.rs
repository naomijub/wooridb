use crate::model::{error::Error, DataRegister};
use crate::repository::local::{LocalContext, UniquenessContext};
use crate::{
    actors::{
        state::{MatchUpdate, PreviousRegistry, State},
        uniques::{CreateUniques, WriteUniques},
        wql::{
            DeleteId, Executor, InsertEntityContent, UpdateContentEntityContent,
            UpdateSetEntityContent,
        },
    },
    model::wql::MatchUpdateArgs,
};
use crate::{
    actors::{
        uniques::CheckForUnique,
        wql::{CreateEntity, EvictEntity, EvictEntityId},
    },
    schemas::tx::EntityResponse,
};

use actix::Addr;
use actix_web::{web, HttpResponse, Responder};
use ron::ser::{to_string_pretty, PrettyConfig};
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
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
    data: web::Data<Arc<Mutex<LocalContext>>>,
    uniqueness: web::Data<Arc<Mutex<UniquenessContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> impl Responder {
    let query = wql::Wql::from_str(&body);
    let response = match query {
        Ok(Wql::CreateEntity(entity, uniques)) => {
            let _ = create_unique_controller(&entity, uniques, uniqueness, &actor).await;
            create_controller(entity, data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::Delete(entity, uuid)) => {
            delete_controller(entity, uuid, data.into_inner(), bytes_counter, actor).await
        }
        Ok(Wql::Insert(entity, content)) => {
            insert_controller(
                entity,
                content,
                data.into_inner(),
                bytes_counter,
                uniqueness,
                actor,
            )
            .await
        }
        Ok(Wql::UpdateContent(entity, content, uuid)) => {
            update_content_controller(
                entity,
                content,
                uuid,
                data.into_inner(),
                bytes_counter,
                uniqueness,
                actor,
            )
            .await
        }
        Ok(Wql::UpdateSet(entity, content, uuid)) => {
            update_set_controller(
                entity,
                content,
                uuid,
                data.into_inner(),
                bytes_counter,
                uniqueness,
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
                actor,
            )
            .await
        }
        Ok(Wql::Evict(entity, uuid)) => {
            evict_controller(entity, uuid, data.into_inner(), bytes_counter, actor).await
        }
        Ok(_) => Err(Error::SelectBadRequest),
        Err(e) => Err(Error::QueryFormat(e)),
    };

    match response {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}

pub async fn create_controller(
    entity: String,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    {
        let mut data = data.lock().unwrap();
        if !data.contains_key(&entity) {
            data.insert(entity.clone(), BTreeMap::new());
        } else {
            return Err(Error::EntityAlreadyCreated(entity));
        }
    }

    let offset = actor
        .send(CreateEntity {
            name: entity.clone(),
        })
        .await
        .unwrap()?;

    bytes_counter.fetch_add(offset, Ordering::SeqCst);

    Ok(EntityResponse::new(entity.clone(), format!("Entity `{}` created", entity)).write())
}

pub async fn evict_controller(
    entity: String,
    uuid: Option<Uuid>,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    if uuid.is_none() {
        let offset = actor
            .send(EvictEntity {
                name: entity.clone(),
            })
            .await
            .unwrap()?;
        bytes_counter.fetch_add(offset, Ordering::SeqCst);

        let mut data = data.lock().unwrap();
        data.remove(&entity);
        Ok(format!("Entity {} evicted", entity))
    } else {
        let id = uuid.unwrap();
        let offset = actor
            .send(EvictEntityId {
                name: entity.clone(),
                id: id.clone(),
            })
            .await
            .unwrap()?;
        bytes_counter.fetch_add(offset, Ordering::SeqCst);

        let mut data = data.lock().unwrap();
        if let Some(d) = data.get_mut(&entity) {
            d.remove(&id);
        }

        Ok(format!("Entity {} with id {} evicted", entity, id))
    }
}

pub async fn create_unique_controller(
    entity: &str,
    uniques: Vec<String>,
    uniqueness: web::Data<Arc<Mutex<UniquenessContext>>>,
    actor: &web::Data<Addr<Executor>>,
) -> Result<(), Error> {
    if uniques.is_empty() {
        Ok(())
    } else {
        let data = uniqueness.into_inner();
        actor
            .send(WriteUniques {
                entity: entity.to_string(),
                uniques: uniques.clone(),
            })
            .await
            .unwrap()?;
        actor
            .send(CreateUniques {
                entity: entity.to_string(),
                uniques,
                data,
            })
            .await
            .unwrap()?;
        Ok(())
    }
}

pub async fn insert_controller(
    entity: String,
    content: HashMap<String, Types>,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    uniqueness: web::Data<Arc<Mutex<UniquenessContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let offset = bytes_counter.load(Ordering::SeqCst);
    let content_log =
        to_string_pretty(&content, pretty_config()).map_err(Error::SerializationError)?;

    let mut data = data.lock().unwrap();
    if !data.contains_key(&entity) {
        return Err(Error::EntityNotCreated(entity));
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUnique {
            entity: entity.clone(),
            content,
            uniqueness,
        })
        .await
        .unwrap()?;

    let content_value = actor
        .send(InsertEntityContent {
            name: entity.clone(),
            content: content_log,
        })
        .await
        .unwrap()?;
    let data_register = DataRegister {
        offset,
        bytes_length: content_value.2,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&entity) {
        map.insert(content_value.1, data_register);
    }

    bytes_counter.fetch_add(content_value.2, Ordering::SeqCst);

    Ok(format!(
        "Entity {} inserted with Uuid {}",
        entity, content_value.1
    ))
}

pub async fn update_set_controller(
    entity: String,
    content: HashMap<String, Types>,
    id: Uuid,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    uniqueness: web::Data<Arc<Mutex<UniquenessContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let offset = bytes_counter.load(Ordering::SeqCst);
    let content_log =
        to_string_pretty(&content, pretty_config()).map_err(Error::SerializationError)?;

    let mut data = data.lock().unwrap();
    if !data.contains_key(&entity) {
        return Err(Error::EntityNotCreated(entity));
    } else if data.contains_key(&entity) && !data.get(&entity).unwrap().contains_key(&id) {
        return Err(Error::UuidNotCreatedForEntity(entity, id));
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUnique {
            entity: entity.clone(),
            content: content.clone(),
            uniqueness,
        })
        .await
        .unwrap()?;

    let previous_entry = data.get(&entity).unwrap().get(&id).unwrap();
    let previous_state_str = actor.send(previous_entry.clone()).await.unwrap()?;
    let mut previous_state = actor
        .send(State(previous_state_str.clone()))
        .await
        .unwrap()?;

    content.into_iter().for_each(|(k, v)| {
        let local_state = previous_state.entry(k).or_insert_with(|| v.clone());
        *local_state = v;
    });

    let state_log =
        to_string_pretty(&previous_state, pretty_config()).map_err(Error::SerializationError)?;

    let content_value = actor
        .send(UpdateSetEntityContent {
            name: entity.clone(),
            current_state: state_log,
            content_log,
            id,
            previous_registry: to_string_pretty(&previous_entry.clone(), pretty_config())
                .map_err(Error::SerializationError)?,
        })
        .await
        .unwrap()?;

    let data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&entity) {
        if let Some(reg) = map.get_mut(&id) {
            *reg = data_register;
        }
    }

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);

    Ok(format!("Entity {} with Uuid {} updated", entity, id))
}

pub async fn update_content_controller(
    entity: String,
    content: HashMap<String, Types>,
    id: Uuid,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    uniqueness: web::Data<Arc<Mutex<UniquenessContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let offset = bytes_counter.load(Ordering::SeqCst);
    let content_log =
        to_string_pretty(&content, pretty_config()).map_err(Error::SerializationError)?;

    let mut data = data.lock().unwrap();
    if !data.contains_key(&entity) {
        return Err(Error::EntityNotCreated(entity));
    } else if data.contains_key(&entity) && !data.get(&entity).unwrap().contains_key(&id) {
        return Err(Error::UuidNotCreatedForEntity(entity, id));
    }

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUnique {
            entity: entity.clone(),
            content: content.clone(),
            uniqueness,
        })
        .await
        .unwrap()?;

    let previous_entry = data.get(&entity).unwrap().get(&id).unwrap();
    let previous_state_str = actor.send(previous_entry.clone()).await.unwrap()?;
    let mut previous_state = actor
        .send(State(previous_state_str.clone()))
        .await
        .unwrap()?;

    content.into_iter().for_each(|(k, v)| {
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
            }
            Types::String(s) => {
                if let Types::String(local) = local_state {
                    *local_state = Types::String(local.to_string() + &s);
                }
            }
            Types::Uuid(uuid) => {
                *local_state = Types::Uuid(uuid);
            }
            Types::Float(f) => {
                if let Types::Float(local) = *local_state {
                    *local_state = Types::Float(local + f);
                }
            }
            Types::Boolean(b) => {
                *local_state = Types::Boolean(b);
            }
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
                            .entry(key.to_string())
                            .and_modify(|v| *v = value.to_owned())
                            .or_insert_with(|| value.to_owned());
                    });
                    *local_state = Types::Map(local.to_owned());
                }
            }
            Types::Nil => {
                *local_state = Types::Nil;
            }
        }
    });

    let state_log =
        to_string_pretty(&previous_state, pretty_config()).map_err(Error::SerializationError)?;

    let content_value = actor
        .send(UpdateContentEntityContent {
            name: entity.clone(),
            current_state: state_log,
            content_log,
            id,
            previous_registry: to_string_pretty(&previous_entry.clone(), pretty_config())
                .map_err(Error::SerializationError)?,
        })
        .await
        .unwrap()?;

    let data_register = DataRegister {
        offset,
        bytes_length: content_value.1,
        file_name: content_value.0.format("%Y_%m_%d.log").to_string(),
    };

    if let Some(map) = data.get_mut(&entity) {
        if let Some(reg) = map.get_mut(&id) {
            *reg = data_register;
        }
    }

    bytes_counter.fetch_add(content_value.1, Ordering::SeqCst);

    Ok(format!("Entity {} with Uuid {} updated", entity, id))
}

pub async fn delete_controller(
    entity: String,
    id: String,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let uuid = Uuid::from_str(&id).unwrap();
    let offset = bytes_counter.load(Ordering::SeqCst);

    let mut data = data.lock().unwrap();
    if !data.contains_key(&entity) {
        return Err(Error::EntityNotCreated(entity));
    } else if data.contains_key(&entity) && !data.get(&entity).unwrap().contains_key(&uuid) {
        return Err(Error::UuidNotCreatedForEntity(entity, uuid));
    }

    let previous_entry = data.get(&entity).unwrap().get(&uuid).unwrap();
    let previous_state_str = actor.send(previous_entry.clone()).await.unwrap()?;
    let two_registries_ago = actor
        .send(PreviousRegistry(previous_state_str))
        .await
        .unwrap()?;

    let state_to_be = match two_registries_ago {
        Some(reg) => {
            let state_str = actor.send(reg.clone()).await.unwrap()?;
            (
                actor.send(State(state_str.clone())).await.unwrap()?,
                reg.to_owned(),
            )
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
        .send(DeleteId {
            name: entity.clone(),
            content_log,
            uuid,
            previous_registry: previous_register_log,
        })
        .await
        .unwrap()?;

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

    Ok(format!("Entity {} with Uuid {} deleted", entity, id))
}

pub async fn match_update_set_controller(
    args: MatchUpdateArgs,
    data: Arc<Arc<Mutex<LocalContext>>>,
    bytes_counter: web::Data<AtomicUsize>,
    uniqueness: web::Data<Arc<Mutex<UniquenessContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let mut data = data.lock().unwrap();
    if !data.contains_key(&args.entity) {
        return Err(Error::EntityNotCreated(args.entity));
    } else if data.contains_key(&args.entity)
        && !data.get(&args.entity).unwrap().contains_key(&args.id)
    {
        return Err(Error::UuidNotCreatedForEntity(args.entity, args.id));
    }

    let previous_entry = data.get(&args.entity).unwrap().get(&args.id).unwrap();
    let previous_state_str = actor.send(previous_entry.clone()).await.unwrap()?;
    let mut previous_state = actor
        .send(State(previous_state_str.clone()))
        .await
        .unwrap()?;

    actor
        .send(MatchUpdate {
            conditions: args.conditions,
            previous_state: previous_state.clone(),
        })
        .await
        .unwrap()?;

    let offset = bytes_counter.load(Ordering::SeqCst);
    let content_log =
        to_string_pretty(&args.content, pretty_config()).map_err(Error::SerializationError)?;

    let uniqueness = uniqueness.into_inner();
    actor
        .send(CheckForUnique {
            entity: args.entity.clone(),
            content: args.content.clone(),
            uniqueness,
        })
        .await
        .unwrap()?;

    args.content.into_iter().for_each(|(k, v)| {
        let local_state = previous_state.entry(k).or_insert_with(|| v.clone());
        *local_state = v;
    });

    let state_log =
        to_string_pretty(&previous_state, pretty_config()).map_err(Error::SerializationError)?;

    let content_value = actor
        .send(UpdateSetEntityContent {
            name: args.entity.clone(),
            current_state: state_log,
            content_log,
            id: args.id,
            previous_registry: to_string_pretty(&previous_entry.clone(), pretty_config())
                .map_err(Error::SerializationError)?,
        })
        .await
        .unwrap()?;

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

    Ok(format!(
        "Entity {} with Uuid {} updated",
        args.entity, args.id
    ))
}
