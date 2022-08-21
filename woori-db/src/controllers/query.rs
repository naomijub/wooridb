use std::{
    collections::{BTreeMap, HashMap, HashSet},
    str::FromStr,
};

use actix_web::{HttpResponse, Responder};
use log::debug;
use rayon::prelude::*;
use uuid::Uuid;
use wql::{ToSelect, Types, Wql};

use crate::{
    actors::{
        encrypts::VerifyEncryption,
        state::State,
        when::{ReadEntitiesAt, ReadEntityIdAt, ReadEntityRange},
    },
    core::query::{
        dedup_option_states, dedup_states, filter_keys_and_hash, get_limit_offset_count,
        get_result_after_manipulation, get_result_after_manipulation_for_options,
        registries_to_states,
    },
    model::{
        error::{error_to_http, Error},
        DataEncryptContext, DataExecutor, DataLocalContext, DataRegister,
    },
    schemas::query::Response as QueryResponse,
};

use super::{
    clauses::select_where_controller,
    relation::{difference, intersect, join, union},
};

pub async fn wql_handler(
    body: String,
    local_data: DataLocalContext,
    encryption: DataEncryptContext,
    actor: DataExecutor,
) -> impl Responder {
    let query = Wql::from_str(&body);
    let response = match query {
        Ok(Wql::Select(entity, ToSelect::All, Some(uuid), _)) => {
            select_all_with_id(entity, uuid, local_data).await
        }
        Ok(Wql::Select(entity, ToSelect::Keys(keys), Some(uuid), _)) => {
            select_keys_with_id(entity, uuid, keys, local_data).await
        }
        Ok(Wql::Select(entity, ToSelect::All, None, functions)) => {
            debug! ("wql_handler will call select_all");
            select_all(entity, local_data, functions).await
        }
        Ok(Wql::Select(entity, ToSelect::Keys(keys), None, functions)) => {
            select_args(entity, keys, local_data, functions).await
        }
        Ok(Wql::SelectIds(entity, ToSelect::All, uuids, functions)) => {
            select_all_with_ids(entity, uuids, local_data, functions).await
        }
        Ok(Wql::SelectIds(entity, ToSelect::Keys(keys), uuids, functions)) => {
            select_keys_with_ids(entity, keys, uuids, local_data, functions).await
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
        Ok(Wql::SelectWhere(entity_name, args_to_select, clauses, functions)) => {
            select_where_controller(entity_name, args_to_select, clauses, local_data, functions)
                .await
        }
        Ok(Wql::CheckValue(entity, uuid, content)) => {
            check_value_controller(entity, uuid, content, local_data, encryption, actor).await
        }
        Ok(Wql::RelationQuery(queries, wql::Relation::Intersect, relation_type)) => {
            intersect(queries, relation_type, local_data, actor).await
        }
        Ok(Wql::RelationQuery(queries, wql::Relation::Difference, relation_type)) => {
            difference(queries, relation_type, local_data, actor).await
        }
        Ok(Wql::RelationQuery(queries, wql::Relation::Union, relation_type)) => {
            union(queries, relation_type, local_data, actor).await
        }
        Ok(Wql::Join(entity_a, entity_b, queries)) => {
            join(entity_a, entity_b, queries, local_data).await
        }
        Ok(_) => Err(Error::NonSelectQuery),
        Err(e) => Err(Error::QueryFormat(e)),
    };

    match response {
        Err(e) => error_to_http(&e),
        Ok(resp) => match resp.to_string() {
            Ok(body) => HttpResponse::Ok().body(body),
            Err(e) => error_to_http(&e),
        },
    }
}

pub async fn check_value_controller(
    entity: String,
    uuid: Uuid,
    content: HashMap<String, String>,
    local_data: DataLocalContext,
    encryption: DataEncryptContext,
    actor: DataExecutor,
) -> Result<QueryResponse, Error> {
    if let Ok(guard) = encryption.lock() {
        if guard.contains_key(&entity) {
            let encrypts = guard.get(&entity).unwrap();
            let non_encrypt_keys = content
                .par_iter()
                .filter(|(k, _)| !encrypts.contains(&(*k).to_string()))
                .map(|(k, _)| k.to_owned())
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
    let previous_state_str = actor.send(previous_entry.0.to_owned()).await??;
    let state = actor.send(State(previous_state_str)).await??;
    let keys = content
        .keys()
        .map(ToOwned::to_owned)
        .collect::<HashSet<String>>();
    let filtered_state: HashMap<String, Types> = state
        .into_par_iter()
        .filter(|(k, _)| keys.contains(k))
        .collect();
    let results = actor
        .send(VerifyEncryption::new(filtered_state, content))
        .await??;
    Ok(results)
}

async fn select_all_when_range_controller(
    entity: String,
    uuid: Uuid,
    start_date: String,
    end_date: String,
    actor: DataExecutor,
) -> Result<QueryResponse, Error> {
    use chrono::{DateTime, Utc};
    let start_date: DateTime<Utc> = start_date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    let end_date: DateTime<Utc> = end_date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    #[cfg(test)]
    let date_log = start_date.format("data/%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = start_date.format("data/%Y_%m_%d.log").to_string();

    let result = actor
        .send(ReadEntityRange::new(
            &entity, uuid, start_date, end_date, date_log,
        ))
        .await??;

    Ok(result.into())
}
async fn select_all_when_controller(
    entity: String,
    date: String,
    actor: DataExecutor,
) -> Result<QueryResponse, Error> {
    use chrono::{DateTime, Utc};
    let date = date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    #[cfg(test)]
    let date_log = date.format("data/%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = date.format("data/%Y_%m_%d.log").to_string();
    let result = actor
        .send(ReadEntitiesAt::new(&entity, date_log, None))
        .await??;

    Ok(result.into())
}

pub async fn select_all_id_when_controller(
    entity: String,
    date: String,
    uuid: Uuid,
    actor: DataExecutor,
) -> Result<QueryResponse, Error> {
    use chrono::{DateTime, Utc};
    let date = date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    #[cfg(test)]
    let date_log = date.format("data/%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = date.format("data/%Y_%m_%d.log").to_string();
    let result = actor
        .send(ReadEntityIdAt::new(&entity, uuid, date_log))
        .await??;
    let result = filter_keys_and_hash(result, None);
    Ok(result.into())
}

pub async fn select_keys_id_when_controller(
    entity: String,
    date: String,
    keys: Vec<String>,
    uuid: Uuid,
    actor: DataExecutor,
) -> Result<QueryResponse, Error> {
    use chrono::{DateTime, Utc};
    let keys = keys.into_par_iter().collect::<HashSet<String>>();
    let date = date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;
    #[cfg(test)]
    let date_log = date.format("data/%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = date.format("data/%Y_%m_%d.log").to_string();
    let result = actor
        .send(ReadEntityIdAt::new(&entity, uuid, date_log))
        .await??;
    let result = filter_keys_and_hash(result, Some(keys));

    Ok(result.into())
}

async fn select_keys_when_controller(
    entity: String,
    date: String,
    keys: Vec<String>,
    actor: DataExecutor,
) -> Result<QueryResponse, Error> {
    use chrono::{DateTime, Utc};
    let keys = keys.into_par_iter().collect::<HashSet<String>>();
    let date = date
        .parse::<DateTime<Utc>>()
        .map_err(Error::DateTimeParse)?;

    #[cfg(test)]
    let date_log = date.format("data/%Y_%m_%d.txt").to_string();
    #[cfg(not(test))]
    let date_log = date.format("data/%Y_%m_%d.log").to_string();
    let result = actor
        .send(ReadEntitiesAt::new(&entity, date_log, Some(keys)))
        .await??;

    Ok(result.into())
}

pub async fn select_all_with_id(
    entity: String,
    uuid: Uuid,
    local_data: DataLocalContext,
) -> Result<QueryResponse, Error> {
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

    let state: HashMap<String, Types> = bincode::deserialize(&registry.1).unwrap();
    let filtered_state = filter_keys_and_hash(state, None);
    Ok(filtered_state.into())
}

pub async fn select_all_with_ids(
    entity: String,
    uuids: Vec<Uuid>,
    local_data: DataLocalContext,
    functions: HashMap<String, wql::Algebra>,
) -> Result<QueryResponse, Error> {
    let (limit, offset, count) = get_limit_offset_count(&functions);
    let registries = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let registries = if let Some(id_to_registry) = local_data.get(&entity) {
            uuids
                .into_par_iter()
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
                .collect::<Vec<(Uuid, Option<(DataRegister, Vec<u8>)>)>>()
        } else {
            return Err(Error::EntityNotCreated(entity));
        };
        registries
    };

    let mut states: BTreeMap<Uuid, Option<HashMap<String, Types>>> = BTreeMap::new();
    for (uuid, registry) in registries.into_iter().skip(offset).take(limit) {
        if let Some((_, state)) = registry {
            let state: HashMap<String, Types> = bincode::deserialize(&state).unwrap();
            let filtered = filter_keys_and_hash(state, None);
            states.insert(uuid, Some(filtered));
        } else {
            states.insert(uuid, None);
        }
    }

    let states = dedup_option_states(states, &functions);

    Ok(get_result_after_manipulation_for_options(
        states, &functions, count,
    ))
}

pub async fn select_keys_with_id(
    entity: String,
    uuid: Uuid,
    keys: Vec<String>,
    local_data: DataLocalContext,
) -> Result<QueryResponse, Error> {
    let keys = keys.into_par_iter().collect::<HashSet<String>>();
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

    let state: HashMap<String, Types> = bincode::deserialize(&registry.1).unwrap();
    let filtered = filter_keys_and_hash(state, Some(keys));
    Ok(filtered.into())
}

pub async fn select_keys_with_ids(
    entity: String,
    keys: Vec<String>,
    uuids: Vec<Uuid>,
    local_data: DataLocalContext,
    functions: HashMap<String, wql::Algebra>,
) -> Result<QueryResponse, Error> {
    let keys = keys.into_par_iter().collect::<HashSet<String>>();
    let (limit, offset, count) = get_limit_offset_count(&functions);
    let registries = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let registries = if let Some(id_to_registry) = local_data.get(&entity) {
            uuids
                .into_par_iter()
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
                .collect::<Vec<(Uuid, Option<(DataRegister, Vec<u8>)>)>>()
        } else {
            return Err(Error::EntityNotCreated(entity));
        };
        registries
    };

    let mut states: BTreeMap<Uuid, Option<HashMap<String, Types>>> = BTreeMap::new();
    for (uuid, registry) in registries.into_iter().skip(offset).take(limit) {
        if let Some((_, state)) = registry {
            let state: HashMap<String, Types> = bincode::deserialize(&state).unwrap();
            let filtered = filter_keys_and_hash(state, Some(keys.clone()));
            states.insert(uuid, Some(filtered));
        } else {
            states.insert(uuid, None);
        }
    }

    let states = dedup_option_states(states, &functions);

    Ok(get_result_after_manipulation_for_options(
        states, &functions, count,
    ))
}

pub async fn select_all(
    entity: String,
    local_data: DataLocalContext,
    functions: HashMap<String, wql::Algebra>,
) -> Result<QueryResponse, Error> {
    let (limit, offset, count) = get_limit_offset_count(&functions);

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
        debug! ("select_all returning registries: {:?}", registries);
        registries
    };

    let states = registries_to_states(registries, None, offset, limit);
    let states = dedup_states(states, &functions);

    Ok(get_result_after_manipulation(states, &functions, count))
}

pub async fn select_args(
    entity: String,
    keys: Vec<String>,
    local_data: DataLocalContext,
    functions: HashMap<String, wql::Algebra>,
) -> Result<QueryResponse, Error> {
    let (limit, offset, count) = get_limit_offset_count(&functions);
    let keys = keys.into_par_iter().collect::<HashSet<String>>();
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

    let states = registries_to_states(registries, Some(keys), offset, limit);
    let states = dedup_states(states, &functions);
    Ok(get_result_after_manipulation(states, &functions, count))
}
