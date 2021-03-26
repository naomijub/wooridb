use std::collections::HashMap;

use wql::{RelationType, ToSelect, Types, Wql};

use crate::{
    model::{error::Error, DataExecutor, DataLocalContext},
    schemas::query::Response,
};

use super::query::{
    select_all_id_when_controller, select_all_with_id, select_keys_id_when_controller,
    select_keys_with_id,
};

const ERROR: &str = "Only single value queries are allowed, so key `ID` is required and keys `WHEN AT` are optional";

pub async fn intersect(
    queries: Vec<Wql>,
    relation_type: RelationType,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<Response, Error> {
    let first = queries[0].clone();
    let second = queries[1].clone();
    let first = get_query_value(first, local_data.clone(), actor.clone()).await?;
    let second = get_query_value(second, local_data, actor).await?;

    match (first, second, relation_type) {
        (Response::Id(f), Response::Id(s), RelationType::Key) => {
            let mut state = HashMap::new();
            for (k, _) in s {
                if f.contains_key(&k) {
                    let v = f.get(&k).unwrap().to_owned();
                    state.insert(k, v);
                }
            }
            Ok(Response::Intersect(state))
        }
        (Response::Id(f), Response::Id(s), RelationType::KeyValue) => {
            let mut state = HashMap::new();
            for (k, v) in s.clone() {
                if f.contains_key(&k) && f.get(&k) == s.get(&k) {
                    state.insert(k, v);
                }
            }
            Ok(Response::Intersect(state))
        }
        _ => Err(Error::InvalidQuery),
    }
}

pub async fn difference(
    queries: Vec<Wql>,
    relation_type: RelationType,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<Response, Error> {
    let first = queries[0].clone();
    let second = queries[1].clone();
    let first = get_query_value(first, local_data.clone(), actor.clone()).await?;
    let second = get_query_value(second, local_data, actor).await?;
    match (first, second, relation_type) {
        (Response::Id(f), Response::Id(s), RelationType::Key) => {
            let mut state = f.clone();
            for (k, _) in s {
                if f.contains_key(&k) {
                    state.remove(&k);
                }
            }
            Ok(Response::Difference(state))
        }
        (Response::Id(f), Response::Id(s), RelationType::KeyValue) => {
            let mut state = f
                .iter()
                .map(|(k, v)| (format!("a:{}", k), v.to_owned()))
                .collect::<HashMap<String, Types>>();
            for (k, _) in s.clone() {
                if f.contains_key(&k) && f.get(&k) == s.get(&k) {
                    state.remove(&k);
                }
            }
            Ok(Response::Difference(state))
        }
        _ => Err(Error::InvalidQuery),
    }
}

pub async fn union(
    queries: Vec<Wql>,
    relation_type: RelationType,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<Response, Error> {
    let first = queries[0].clone();
    let second = queries[1].clone();
    let first = get_query_value(first, local_data.clone(), actor.clone()).await?;
    let second = get_query_value(second, local_data, actor).await?;
    match (first, second, relation_type) {
        (Response::Id(f), Response::Id(s), RelationType::Key) => {
            let mut state = f.clone();
            for (k, v) in s {
                if !f.contains_key(&k) {
                    state.insert(k, v);
                }
            }
            Ok(Response::Union(state))
        }
        (Response::Id(f), Response::Id(s), RelationType::KeyValue) => {
            let mut state = f.clone();
            for (k, v) in s.clone() {
                if f.get(&k) != s.get(&k) && f.get(&k).is_some() {
                    let key = format!("{}:duplicated", k);
                    state.insert(key, v);
                } else if f.get(&k) != s.get(&k) && f.get(&k).is_none() {
                    let value = s.get(&k).unwrap().to_owned();
                    state.insert(k, value);
                }
            }
            Ok(Response::Union(state))
        }
        _ => Err(Error::InvalidQuery),
    }
}

async fn get_query_value(
    query: Wql,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<Response, Error> {
    match query {
        Wql::Select(entity, ToSelect::All, Some(uuid), _) => {
            select_all_with_id(entity, uuid, local_data).await
        }
        Wql::Select(entity, ToSelect::Keys(keys), Some(uuid), _) => {
            select_keys_with_id(entity, uuid, keys, local_data).await
        }
        Wql::SelectWhen(entity, ToSelect::All, Some(uuid), date) => {
            select_all_id_when_controller(entity, date, uuid, actor).await
        }
        Wql::SelectWhen(entity, ToSelect::Keys(keys), Some(uuid), date) => {
            select_keys_id_when_controller(entity, date, keys, uuid, actor).await
        }
        _ => Err(Error::QueryFormat(String::from(ERROR))),
    }
}
