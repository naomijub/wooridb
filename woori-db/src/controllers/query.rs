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
    model::error::Error,
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
