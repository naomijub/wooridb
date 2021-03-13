use crate::model::error::{Error, error_to_http};
use crate::{
    actors::history::History,
    core::pretty_config_output,
    model::{DataExecutor, DataLocalContext},
    schemas::history::EntityHistoryInfo,
};

use actix_web::{HttpResponse, Responder};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap};
use wql::Types;

pub async fn history_handler(
    body: String,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> impl Responder {
    let response = history_controller(body, local_data, actor);

    match response.await {
        Err(e) => error_to_http(e),
        Ok(resp) => HttpResponse::Ok().body(resp),
    }
}

pub async fn history_controller(
    body: String,
    local_data: DataLocalContext,
    actor: DataExecutor,
) -> Result<String, Error> {
    let info: EntityHistoryInfo = ron::de::from_str(&body)?;

    let registry = {
        let local_data = if let Ok(guard) = local_data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };
        let registry = if let Some(id_to_registry) = local_data.get(&info.entity_key) {
            if let Some(reg) = id_to_registry.get(&info.entity_id) {
                reg
            } else {
                return Err(Error::UuidNotCreatedForEntity(
                    info.entity_key,
                    info.entity_id,
                ));
            }
        } else {
            return Err(Error::EntityNotCreated(info.entity_key));
        }
        .to_owned();
        registry
    };
    let mut content = actor.send(registry.0).await??;
    let mut btree: BTreeMap<DateTime<Utc>, HashMap<String, Types>> = BTreeMap::new();

    loop {
        let (entity_map, date, previous_registry) = actor.send(History(content.clone())).await??;
        if let (Some(start), Some(end)) = (info.start_datetime, info.end_datetime) {
            if date >= start && date <= end {
                btree.insert(date, entity_map);
            } else if date > end {
                break;
            }
        } else if let (Some(start), None) = (info.start_datetime, info.end_datetime) {
            if date >= start {
                btree.insert(date, entity_map);
            }
        } else if let (None, Some(end)) = (info.start_datetime, info.end_datetime) {
            if date <= end {
                btree.insert(date, entity_map);
            } else if date > end {
                break;
            }
        } else {
            btree.insert(date, entity_map);
        }

        if previous_registry.is_none() {
            break;
        }
        content = actor.send(previous_registry.unwrap()).await??;
    }

    let filtered_tree = btree
        .into_par_iter()
        .map(|(date, content)| {
            (
                date,
                content
                    .into_iter()
                    .filter(|(_, v)| !v.is_hash())
                    .collect::<HashMap<String, Types>>(),
            )
        })
        .collect::<BTreeMap<DateTime<Utc>, HashMap<String, Types>>>();

    Ok(ron::ser::to_string_pretty(
        &filtered_tree,
        pretty_config_output(),
    )?)
}
