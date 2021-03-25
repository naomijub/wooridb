use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use wql::Types;

use crate::{
    actors::wql::{
        DeleteId, EvictEntityId, InsertEntityContent, UpdateContentEntityContent,
        UpdateSetEntityContent,
    },
    core::pretty_config_inner,
    model::wql::Action,
};
use ron::ser::to_string_pretty;

pub fn create_entity(entity: &str) -> String {
    format!("{}|{};", Action::CreateEntity, entity)
}

pub fn evict_entity_content(entity: &str) -> String {
    let date: DateTime<Utc> = Utc::now();
    let date = to_string_pretty(&date, pretty_config_inner()).unwrap();
    format!("{}|{}|{};", Action::EvictEntity, date, entity)
}

pub fn evict_entity_id_content(entity: &EvictEntityId) -> String {
    let date: DateTime<Utc> = Utc::now();
    let date = to_string_pretty(&date, pretty_config_inner()).unwrap();
    format!(
        "{}|{}|{}|{};",
        Action::EvictEntityId,
        date,
        entity.id,
        entity.name
    )
}

pub fn insert_entity_content(content: &InsertEntityContent) -> (DateTime<Utc>, Uuid, String) {
    let uuid = content.uuid.map_or_else(|| Uuid::new_v4(), |id| id);

    let date = content.datetime;
    let date_str = to_string_pretty(&date, pretty_config_inner()).unwrap();
    let log = format!(
        "{}|{}|{}|{}|{};",
        Action::Insert,
        date_str,
        uuid.to_string(),
        content.name,
        content.content
    );
    (date, uuid, log)
}

pub fn update_set_entity_content(content: &UpdateSetEntityContent) -> (DateTime<Utc>, String) {
    let uuid = content.id;
    let date = content.datetime;
    let date_str = to_string_pretty(&date, pretty_config_inner()).unwrap();
    let log = format!(
        "{}|{}|{}|{}|{}|{}|{};",
        Action::UpdateSet,
        date_str,
        uuid.to_string(),
        content.name,
        content.content_log,
        content.current_state,
        content.previous_registry
    );
    (date, log)
}

pub fn update_content_entity_content(
    content: &UpdateContentEntityContent,
) -> (DateTime<Utc>, String) {
    let uuid = content.id;
    let date: DateTime<Utc> = Utc::now();
    let date_str = to_string_pretty(&date, pretty_config_inner()).unwrap();
    let log = format!(
        "{}|{}|{}|{}|{}|{}|{};",
        Action::UpdateContent,
        date_str,
        uuid.to_string(),
        content.name,
        content.content_log,
        content.current_state,
        content.previous_registry
    );
    (date, log)
}

pub fn delete_entity_content(content: &DeleteId) -> (DateTime<Utc>, String) {
    let date: DateTime<Utc> = Utc::now();
    let date_str = to_string_pretty(&date, pretty_config_inner()).unwrap();

    let log = format!(
        "{}|{}|{}|{}|{}|{};",
        Action::Delete,
        date_str,
        content.uuid.to_string(),
        content.name,
        content.content_log,
        content.previous_registry
    );
    (date, log)
}

pub fn update_content_state(previous_state: &mut HashMap<String, Types>, k: String, v: Types) {
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
        Types::DateTime(date) => {
            *local_state = Types::DateTime(date);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::actors::wql::{
        DeleteId, InsertEntityContent, UpdateContentEntityContent, UpdateSetEntityContent,
    };

    #[test]
    fn create_entity_test() {
        let s = create_entity(&"my_entity".to_string());
        assert_eq!(s, "CREATE_ENTITY|my_entity;");
    }

    #[test]
    fn insert_entity_test() {
        let entity = InsertEntityContent {
            name: "my_entity".to_string(),
            content: "suppose this is a log".to_string(),
            uuid: None,
            datetime: Utc::now(),
        };
        let (_, _, s) = insert_entity_content(&entity);

        assert!(s.contains("INSERT"));
        assert!(s.contains("my_entity"));
        assert!(s.contains("suppose this is a log"));
    }

    #[test]
    fn insert_entitywith_uuid_test() {
        let uuid = Uuid::new_v4();
        let entity = InsertEntityContent {
            name: "my_entity".to_string(),
            content: "suppose this is a log".to_string(),
            uuid: Some(uuid),
            datetime: Utc::now(),
        };
        let (_, _, s) = insert_entity_content(&entity);

        assert!(s.contains("INSERT"));
        assert!(s.contains("my_entity"));
        assert!(s.contains(&uuid.to_string()));
        assert!(s.contains("suppose this is a log"));
    }

    #[test]
    fn update_set_entity_content_test() {
        let id = uuid::Uuid::new_v4();
        let entity = UpdateSetEntityContent {
            name: "my-entity".to_string(),
            current_state: "state".to_string(),
            content_log: "log".to_string(),
            id,
            datetime: Utc::now(),
            previous_registry: "reg".to_string(),
        };

        let (_, s) = update_set_entity_content(&entity);
        assert!(s.contains("UPDATE_SET"));
        assert!(s.contains("my-entity"));
        assert!(s.contains("state"));
        assert!(s.contains("log"));
        assert!(s.contains("reg"));
    }

    #[test]
    fn update_content_entity_content_test() {
        let id = uuid::Uuid::new_v4();
        let entity = UpdateContentEntityContent {
            name: "my-entity".to_string(),
            current_state: "state".to_string(),
            content_log: "log".to_string(),
            id,
            previous_registry: "reg".to_string(),
        };

        let (_, s) = update_content_entity_content(&entity);
        assert!(s.contains("UPDATE_CONTENT"));
        assert!(s.contains("my-entity"));
        assert!(s.contains("state"));
        assert!(s.contains("log"));
        assert!(s.contains("reg"));
    }

    #[test]
    fn delete_entity_test() {
        let id = uuid::Uuid::new_v4();
        let entity = DeleteId {
            name: "my-entity".to_string(),
            content_log: "log".to_string(),
            uuid: id,
            previous_registry: "reg".to_string(),
        };

        let (_, s) = delete_entity_content(&entity);
        assert!(s.contains("DELETE"));
        assert!(s.contains("my-entity"));
        assert!(s.contains("log"));
        assert!(s.contains("reg"));
    }

    #[test]
    fn evict_entity_test() {
        let entity = "hello";

        let actual = evict_entity_content(&entity);

        assert!(actual.starts_with("EVICT_ENTITY"));
        assert!(actual.contains("hello"))
    }

    #[test]
    fn evict_entity_id_test() {
        let uuid = Uuid::new_v4();
        let entity = EvictEntityId {
            name: "hello".to_string(),
            id: uuid,
        };

        let actual = evict_entity_id_content(&entity);

        assert!(actual.starts_with("EVICT_ENTITY_ID"));
        assert!(actual.contains("hello"));
        assert!(actual.contains(&uuid.to_string()));
    }
}
