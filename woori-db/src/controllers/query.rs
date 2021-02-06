use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::{Arc, Mutex},
};

use actix::Addr;
use actix_web::{web, HttpResponse, Responder};
use ron::ser::PrettyConfig;
use uuid::Uuid;
use wql::{ToSelect, Types, Wql};

use crate::{
    actors::{state::State, wql::Executor},
    model::{error::Error, DataRegister},
    repository::local::LocalContext,
};

fn pretty_config() -> PrettyConfig {
    PrettyConfig::new()
        .with_separate_tuple_members(true)
        .with_decimal_floats(true)
        .with_indentor(" ".to_string())
        .with_new_line("\n".to_string())
}

pub async fn wql_handler(
    body: String,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> impl Responder {
    let query = Wql::from_str(&body);
    let response = match query {
        Ok(Wql::Select(entity, ToSelect::All, Some(uuid))) => {
            select_all_with_id(entity, uuid, data, actor).await
        }
        Ok(Wql::Select(entity, ToSelect::Keys(keys), Some(uuid))) => {
            select_keys_with_id(entity, uuid, keys, data, actor).await
        }
        Ok(Wql::Select(entity, ToSelect::All, None)) => select_all(entity, data, actor).await,
        Ok(Wql::Select(entity, ToSelect::Keys(keys), None)) => {
            select_args(entity, keys, data, actor).await
        }
        Ok(Wql::SelectIds(entity, ToSelect::All, uuids)) => {
            select_all_with_ids(entity, uuids, data, actor).await
        }
        Ok(Wql::SelectIds(entity, ToSelect::Keys(keys), uuids)) => {
            select_keys_with_ids(entity, keys, uuids, data, actor).await
        }
        Ok(_) => Err(Error::NonSelectQuery),
        Err(e) => Err(Error::QueryFormat(e)),
    };

    match response {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}

async fn select_all_with_id(
    entity: String,
    uuid: Uuid,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let data = data.lock().unwrap();
    let registry = if let Some(id_to_registry) = data.get(&entity) {
        if let Some(reg) = id_to_registry.get(&uuid) {
            reg
        } else {
            return Err(Error::UuidNotCreatedForEntity(entity, uuid));
        }
    } else {
        return Err(Error::EntityNotCreated(entity));
    }
    .clone();

    let content = actor.send(registry).await.unwrap()?;
    let state = actor.send(State(content)).await.unwrap()?;
    Ok(ron::ser::to_string_pretty(&state, pretty_config()).unwrap())
}

async fn select_all_with_ids(
    entity: String,
    uuids: Vec<Uuid>,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let data = data.lock().unwrap();
    let registries = if let Some(id_to_registry) = data.get(&entity) {
        uuids
            .into_iter()
            .filter_map(|id| {
                Some((
                    id,
                    id_to_registry
                        .get(&id)
                        .ok_or_else(|| Error::UuidNotCreatedForEntity(entity.clone(), id.clone()))
                        .ok(),
                ))
                .filter(|(_id, reg)| reg.is_some())
            })
            .map(|(uuid, reg)| (uuid, reg.map(|d| d.clone())))
            .collect::<Vec<(Uuid, Option<DataRegister>)>>()
    } else {
        return Err(Error::EntityNotCreated(entity));
    }
    .clone();

    let mut states: HashMap<Uuid, Option<HashMap<String, Types>>> = HashMap::new();
    for (uuid, registry) in registries.into_iter() {
        if let Some(regs) = registry {
            let content = actor.send(regs).await.unwrap()?;
            let state = actor.send(State(content)).await.unwrap()?;
            states.insert(uuid, Some(state));
        } else {
            states.insert(uuid, None);
        }
    }

    Ok(ron::ser::to_string_pretty(&states, pretty_config()).unwrap())
}

async fn select_keys_with_id(
    entity: String,
    uuid: Uuid,
    keys: Vec<String>,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let keys = keys.into_iter().collect::<HashSet<String>>();
    let data = data.lock().unwrap();
    let registry = if let Some(id_to_registry) = data.get(&entity) {
        if let Some(reg) = id_to_registry.get(&uuid) {
            reg
        } else {
            return Err(Error::UuidNotCreatedForEntity(entity, uuid));
        }
    } else {
        return Err(Error::EntityNotCreated(entity));
    }
    .clone();

    let content = actor.send(registry).await.unwrap()?;
    let state = actor.send(State(content)).await.unwrap()?;
    let filtered: HashMap<String, Types> = state
        .into_iter()
        .filter(|(k, _)| keys.contains(k))
        .collect();
    Ok(ron::ser::to_string_pretty(&filtered, pretty_config()).unwrap())
}

async fn select_keys_with_ids(
    entity: String,
    keys: Vec<String>,
    uuids: Vec<Uuid>,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let data = data.lock().unwrap();
    let registries = if let Some(id_to_registry) = data.get(&entity) {
        uuids
            .into_iter()
            .filter_map(|id| {
                Some((
                    id,
                    id_to_registry
                        .get(&id)
                        .ok_or_else(|| Error::UuidNotCreatedForEntity(entity.clone(), id.clone()))
                        .ok(),
                ))
                .filter(|(_id, reg)| reg.is_some())
            })
            .map(|(uuid, reg)| (uuid, reg.map(|d| d.clone())))
            .collect::<Vec<(Uuid, Option<DataRegister>)>>()
    } else {
        return Err(Error::EntityNotCreated(entity));
    }
    .clone();

    let mut states: HashMap<Uuid, Option<HashMap<String, Types>>> = HashMap::new();
    for (uuid, registry) in registries.into_iter() {
        if let Some(regs) = registry {
            let content = actor.send(regs).await.unwrap()?;
            let state = actor.send(State(content)).await.unwrap()?;
            let filtered: HashMap<String, Types> = state
                .into_iter()
                .filter(|(k, _)| keys.contains(k))
                .collect();
            states.insert(uuid, Some(filtered));
        } else {
            states.insert(uuid, None);
        }
    }

    Ok(ron::ser::to_string_pretty(&states, pretty_config()).unwrap())
}

async fn select_all(
    entity: String,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let data = data.lock().unwrap();
    let registries = if let Some(id_to_registries) = data.get(&entity) {
        id_to_registries
    } else {
        return Err(Error::EntityNotCreated(entity));
    }
    .clone();

    let kvs = registries
        .into_iter()
        .map(|k| k)
        .collect::<Vec<(Uuid, DataRegister)>>();

    let mut states: HashMap<Uuid, HashMap<String, Types>> = HashMap::new();
    for (uuid, regs) in kvs.into_iter() {
        let content = actor.send(regs).await.unwrap()?;
        let state = actor.send(State(content)).await.unwrap()?;
        states.insert(uuid, state);
    }

    Ok(ron::ser::to_string_pretty(&states, pretty_config()).unwrap())
}

async fn select_args(
    entity: String,
    keys: Vec<String>,
    data: web::Data<Arc<Mutex<LocalContext>>>,
    actor: web::Data<Addr<Executor>>,
) -> Result<String, Error> {
    let keys = keys.into_iter().collect::<HashSet<String>>();
    let data = data.lock().unwrap();
    let registries = if let Some(id_to_registries) = data.get(&entity) {
        id_to_registries
    } else {
        return Err(Error::EntityNotCreated(entity));
    }
    .clone();

    let kvs = registries
        .into_iter()
        .map(|k| k)
        .collect::<Vec<(Uuid, DataRegister)>>();

    let mut states: HashMap<Uuid, HashMap<String, Types>> = HashMap::new();
    for (uuid, regs) in kvs.into_iter() {
        let content = actor.send(regs).await.unwrap()?;
        let state = actor.send(State(content)).await.unwrap()?;
        let filtered: HashMap<String, Types> = state
            .into_iter()
            .filter(|(k, _)| keys.contains(k))
            .collect();
        states.insert(uuid, filtered);
    }

    Ok(ron::ser::to_string_pretty(&states, pretty_config()).unwrap())
}
