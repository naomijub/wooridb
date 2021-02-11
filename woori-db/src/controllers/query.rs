use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use actix_web::{HttpResponse, Responder};
use ron::ser::to_string_pretty;
use uuid::Uuid;
use wql::{ToSelect, Types, Wql};

use crate::{
    actors::{
        state::State,
        when::{ReadEntitiesAt, ReadEntityIdAt, ReadEntityRange},
    },
    core::pretty_config_output,
    model::{error::Error, DataExecutor, DataLocalContext, DataRegister},
};

use super::clauses::select_where;

pub async fn wql_handler(
    body: String,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> impl Responder {
    let query = Wql::from_str(&body);
    let response = match query {
        Ok(Wql::Select(entity, ToSelect::All, Some(uuid))) => {
            select_all_with_id(entity, uuid, local_data, actor).await
        }
        Ok(Wql::Select(entity, ToSelect::Keys(keys), Some(uuid))) => {
            select_keys_with_id(entity, uuid, keys, local_data, actor).await
        }
        Ok(Wql::Select(entity, ToSelect::All, None)) => select_all(entity, local_data, actor).await,
        Ok(Wql::Select(entity, ToSelect::Keys(keys), None)) => {
            select_args(entity, keys, local_data, actor).await
        }
        Ok(Wql::SelectIds(entity, ToSelect::All, uuids)) => {
            select_all_with_ids(entity, uuids, local_data, actor).await
        }
        Ok(Wql::SelectIds(entity, ToSelect::Keys(keys), uuids)) => {
            select_keys_with_ids(entity, keys, uuids, local_data, actor).await
        }
        Ok(Wql::SelectWhen(entity, ToSelect::All, None, date)) => {
            select_all_when_controller(entity, date, actor).await
        }
        Ok(Wql::SelectWhen(entity, ToSelect::Keys(keys), None, date)) => {
            select_keys_when_controller(entity, date, keys, actor).await
        }
        Ok(Wql::SelectWhen(entity, ToSelect::All, Some(uuid), date)) => {
            select_all_id_when_controller(entity, date, uuid, actor).await
        }
        Ok(Wql::SelectWhen(entity, ToSelect::Keys(keys), Some(uuid), date)) => {
            select_keys_id_when_controller(entity, date, keys, uuid, actor).await
        }
        Ok(Wql::SelectWhenRange(entity_name, uuid, start_date, end_date)) => {
            select_all_when_range_controller(entity_name, uuid, start_date, end_date, actor).await
        }
        Ok(Wql::SelectWhere(entity_name, args_to_select, clauses)) => {
            select_where(entity_name, args_to_select, clauses, local_data, actor).await
        }
        Ok(_) => Err(Error::NonSelectQuery),
        Err(e) => Err(Error::QueryFormat(e)),
    };

    match response {
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}

async fn select_all_when_range_controller(
    entity: String,
    uuid: Uuid,
    start_date: String,
    end_date: String,
    actor: DataExecutor,
) -> Result<String, Error> {
    use chrono::{DateTime, Utc};
    let start_date: DateTime<Utc> = start_date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    let end_date: DateTime<Utc> = end_date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    #[cfg(test)]
    let date_log = start_date.format("%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = start_date.format("%Y_%m_%d.log").to_string();

    let result = actor
        .send(ReadEntityRange::new(
            &entity, uuid, start_date, end_date, date_log,
        ))
        .await??;

    Ok(to_string_pretty(&result, pretty_config_output())?)
}
async fn select_all_when_controller(
    entity: String,
    date: String,
    actor: DataExecutor,
) -> Result<String, Error> {
    use chrono::{DateTime, Utc};
    let date = date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    #[cfg(test)]
    let date_log = date.format("%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = date.format("%Y_%m_%d.log").to_string();
    let result = actor.send(ReadEntitiesAt::new(&entity, date_log)).await??;

    Ok(to_string_pretty(&result, pretty_config_output())?)
}

async fn select_all_id_when_controller(
    entity: String,
    date: String,
    uuid: Uuid,
    actor: DataExecutor,
) -> Result<String, Error> {
    use chrono::{DateTime, Utc};
    let date = date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    #[cfg(test)]
    let date_log = date.format("%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = date.format("%Y_%m_%d.log").to_string();
    let result = actor
        .send(ReadEntityIdAt::new(&entity, uuid, date_log))
        .await??;

    Ok(to_string_pretty(&result, pretty_config_output())?)
}

async fn select_keys_id_when_controller(
    entity: String,
    date: String,
    keys: Vec<String>,
    uuid: Uuid,
    actor: DataExecutor,
) -> Result<String, Error> {
    use chrono::{DateTime, Utc};
    let date = date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    #[cfg(test)]
    let date_log = date.format("%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = date.format("%Y_%m_%d.log").to_string();
    let result = actor
        .send(ReadEntityIdAt::new(&entity, uuid, date_log))
        .await??;
    let result = result
        .into_iter()
        .filter(|(k, _)| keys.contains(k))
        .collect::<HashMap<String, Types>>();

    Ok(to_string_pretty(&result, pretty_config_output())?)
}

async fn select_keys_when_controller(
    entity: String,
    date: String,
    keys: Vec<String>,
    actor: DataExecutor,
) -> Result<String, Error> {
    use chrono::{DateTime, Utc};
    let date = date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;

    #[cfg(test)]
    let date_log = date.format("%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = date.format("%Y_%m_%d.log").to_string();
    let result = actor.send(ReadEntitiesAt::new(&entity, date_log)).await??;
    let result = result
        .into_iter()
        .map(|(id, hm)| {
            (
                id,
                hm.into_iter()
                    .filter(|(k, _)| keys.contains(k))
                    .collect::<HashMap<String, Types>>(),
            )
        })
        .collect::<HashMap<String, HashMap<String, Types>>>();

    Ok(to_string_pretty(&result, pretty_config_output())?)
}

async fn select_all_with_id(
    entity: String,
    uuid: Uuid,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let registry = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let registry = if let Some(id_to_registry) = local_data.get(&entity) {
            if let Some(reg) = id_to_registry.get(&uuid) {
                reg
            } else {
                return Err(Error::UuidNotCreatedForEntity(entity, uuid));
            }
        } else {
            return Err(Error::EntityNotCreated(entity));
        }
        .to_owned();
        registry
    };

    let content = actor.send(registry).await??;
    let state = actor.send(State(content)).await??;
    let filterd_state = state
        .into_iter()
        .filter(|(_, v)| !v.is_hash())
        .collect::<HashMap<String, Types>>();
    Ok(ron::ser::to_string_pretty(
        &filterd_state,
        pretty_config_output(),
    )?)
}

async fn select_all_with_ids(
    entity: String,
    uuids: Vec<Uuid>,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let registries = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let registries = if let Some(id_to_registry) = local_data.get(&entity) {
            uuids
                .into_iter()
                .filter_map(|id| {
                    Some((
                        id,
                        id_to_registry
                            .get(&id)
                            .ok_or_else(|| Error::UuidNotCreatedForEntity(entity.clone(), id))
                            .ok(),
                    ))
                    .filter(|(_id, reg)| reg.is_some())
                })
                .map(|(uuid, reg)| (uuid, reg.map(ToOwned::to_owned)))
                .collect::<Vec<(Uuid, Option<DataRegister>)>>()
        } else {
            return Err(Error::EntityNotCreated(entity));
        };
        registries
    };

    let mut states: HashMap<Uuid, Option<HashMap<String, Types>>> = HashMap::new();
    for (uuid, registry) in registries {
        if let Some(regs) = registry {
            let content = actor.send(regs).await??;
            let state = actor.send(State(content)).await??;
            let filtered = state
                .into_iter()
                .filter(|(_, v)| !v.is_hash())
                .collect::<HashMap<String, Types>>();
            states.insert(uuid, Some(filtered));
        } else {
            states.insert(uuid, None);
        }
    }

    Ok(ron::ser::to_string_pretty(&states, pretty_config_output())?)
}

async fn select_keys_with_id(
    entity: String,
    uuid: Uuid,
    keys: Vec<String>,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let keys = keys.into_iter().collect::<HashSet<String>>();
    let registry = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let registry = if let Some(id_to_registry) = local_data.get(&entity) {
            if let Some(reg) = id_to_registry.get(&uuid) {
                reg
            } else {
                return Err(Error::UuidNotCreatedForEntity(entity, uuid));
            }
        } else {
            return Err(Error::EntityNotCreated(entity));
        }
        .to_owned();
        registry
    };

    let content = actor.send(registry).await??;
    let state = actor.send(State(content)).await??;
    let filtered: HashMap<String, Types> = state
        .into_iter()
        .filter(|(k, _)| keys.contains(k))
        .filter(|(_, v)| !v.is_hash())
        .collect();
    Ok(ron::ser::to_string_pretty(
        &filtered,
        pretty_config_output(),
    )?)
}

async fn select_keys_with_ids(
    entity: String,
    keys: Vec<String>,
    uuids: Vec<Uuid>,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let registries = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let registries = if let Some(id_to_registry) = local_data.get(&entity) {
            uuids
                .into_iter()
                .filter_map(|id| {
                    Some((
                        id,
                        id_to_registry
                            .get(&id)
                            .ok_or_else(|| Error::UuidNotCreatedForEntity(entity.clone(), id))
                            .ok(),
                    ))
                    .filter(|(_id, reg)| reg.is_some())
                })
                .map(|(uuid, reg)| (uuid, reg.map(ToOwned::to_owned)))
                .collect::<Vec<(Uuid, Option<DataRegister>)>>()
        } else {
            return Err(Error::EntityNotCreated(entity));
        };
        registries
    };

    let mut states: HashMap<Uuid, Option<HashMap<String, Types>>> = HashMap::new();
    for (uuid, registry) in registries {
        if let Some(regs) = registry {
            let content = actor.send(regs).await??;
            let state = actor.send(State(content)).await??;
            let filtered: HashMap<String, Types> = state
                .into_iter()
                .filter(|(k, _)| keys.contains(k))
                .filter(|(_, v)| !v.is_hash())
                .collect();
            states.insert(uuid, Some(filtered));
        } else {
            states.insert(uuid, None);
        }
    }

    Ok(ron::ser::to_string_pretty(&states, pretty_config_output())?)
}

async fn select_all(
    entity: String,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let registries = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let registries = if let Some(id_to_registries) = local_data.get(&entity) {
            id_to_registries
        } else {
            return Err(Error::EntityNotCreated(entity));
        }
        .to_owned();
        registries
    };

    let mut states: HashMap<Uuid, HashMap<String, Types>> = HashMap::new();
    for (uuid, regs) in registries {
        let content = actor.send(regs).await??;
        let state = actor.send(State(content)).await??;
        let filtered = state
            .into_iter()
            .filter(|(_, v)| !v.is_hash())
            .collect::<HashMap<String, Types>>();

        states.insert(uuid, filtered);
    }

    Ok(ron::ser::to_string_pretty(&states, pretty_config_output())?)
}

async fn select_args(
    entity: String,
    keys: Vec<String>,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let keys = keys.into_iter().collect::<HashSet<String>>();
    let registries = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let registries = if let Some(id_to_registries) = local_data.get(&entity) {
            id_to_registries
        } else {
            return Err(Error::EntityNotCreated(entity));
        }
        .to_owned();
        registries
    };

    let mut states: HashMap<Uuid, HashMap<String, Types>> = HashMap::new();
    for (uuid, regs) in registries {
        let content = actor.send(regs).await??;
        let state = actor.send(State(content)).await??;
        let filtered: HashMap<String, Types> = state
            .into_iter()
            .filter(|(k, _)| keys.contains(k))
            .filter(|(_, v)| !v.is_hash())
            .collect();
        states.insert(uuid, filtered);
    }

    Ok(ron::ser::to_string_pretty(&states, pretty_config_output())?)
}
