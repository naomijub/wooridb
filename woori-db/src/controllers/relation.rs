use std::collections::HashMap;

use wql::{RelationType, ToSelect, Wql};

use crate::{
    model::{error::Error, DataExecutor, DataLocalContext},
    schemas::query::Response,
};

use super::{
    clauses::select_where_controller,
    query::{
        select_all, select_all_id_when_controller, select_all_with_id, select_all_with_ids,
        select_args, select_keys_id_when_controller, select_keys_with_id, select_keys_with_ids,
    },
};

const ERROR: &str = "Only single value queries are allowed, so key `ID` is required and keys `WHEN AT` are optional";
const ERROR_JOIN: &str =
    "Only multiple values queries are allowed, so key `ID` and `WHEN AT` are not allowed";

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
            let mut state = f.clone();
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

pub async fn join(
    entity_a: (String, String),
    entity_b: (String, String),
    queries: Vec<Wql>,
    local_data: DataLocalContext,
) -> Result<Response, Error> {
    let mut result = Vec::new();
    let a = get_join_query_value(queries[0].clone(), local_data.clone()).await?;
    let b = get_join_query_value(queries[1].clone(), local_data).await?;

    let b_hash = b
        .hash(&entity_b.1)
        .ok_or_else(|| Error::QueryFormat("Join query not supported".to_string()))?;
    let ok = a.parse(entity_a.1, &entity_b, &mut result, b_hash);

    if ok {
        Ok(Response::Join(result))
    } else {
        Err(Error::QueryFormat("Join query not supported".to_string()))
    }
}

async fn get_query_value(
    query: Wql,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<Response, Error> {
    match query {
        Wql::Select(entity, ToSelect::All, Some(id), _) => {
            select_all_with_id(entity, id, local_data).await
        }
        Wql::Select(entity, ToSelect::Keys(keys), Some(id), _) => {
            select_keys_with_id(entity, id, keys, local_data).await
        }
        Wql::SelectWhen(entity, ToSelect::All, Some(id), date) => {
            select_all_id_when_controller(entity, date, id, actor).await
        }
        Wql::SelectWhen(entity, ToSelect::Keys(keys), Some(id), date) => {
            select_keys_id_when_controller(entity, date, keys, id, actor).await
        }
        _ => Err(Error::QueryFormat(String::from(ERROR))),
    }
}

async fn get_join_query_value(query: Wql, local_data: DataLocalContext) -> Result<Response, Error> {
    match query {
        Wql::Select(entity, ToSelect::All, None, functions) => {
            select_all(entity, local_data, functions).await
        }
        Wql::Select(entity, ToSelect::Keys(keys), None, functions) => {
            select_args(entity, keys, local_data, functions).await
        }
        Wql::SelectIds(entity, ToSelect::All, ids, functions) => {
            select_all_with_ids(entity, ids, local_data, functions).await
        }
        Wql::SelectIds(entity, ToSelect::Keys(keys), ids, functions) => {
            select_keys_with_ids(entity, keys, ids, local_data, functions).await
        }
        Wql::SelectWhere(entity_name, args_to_select, clauses, functions) => {
            select_where_controller(entity_name, args_to_select, clauses, local_data, functions)
                .await
        }
        _ => Err(Error::QueryFormat(String::from(ERROR_JOIN))),
    }
}
