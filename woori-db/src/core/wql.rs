use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    actors::wql::{
        DeleteId, InsertEntityContent, UpdateContentEntityContent, UpdateSetEntityContent,
    },
    model::wql::Action,
};

pub fn create_entity(entity: &String) -> String {
    format!("{}|{};", Action::CreateEntity, entity)
}

pub fn insert_entity_content(content: InsertEntityContent) -> (DateTime<Utc>, Uuid, String) {
    let uuid = Uuid::new_v4();
    let date: DateTime<Utc> = Utc::now();

    let log = format!(
        "{}|{}|{}|{}|{};",
        Action::Insert,
        date.to_string(),
        uuid.to_string(),
        content.name,
        content.content
    );
    (date, uuid, log)
}

pub fn update_set_entity_content(content: UpdateSetEntityContent) -> (DateTime<Utc>, String) {
    let uuid = content.id;
    let date: DateTime<Utc> = Utc::now();

    let log = format!(
        "{}|{}|{}|{}|{}|{}|{};",
        Action::UpdateSet,
        date.to_string(),
        uuid.to_string(),
        content.name,
        content.content_log,
        content.current_state,
        content.previous_registry
    );
    (date, log)
}

pub fn update_content_entity_content(
    content: UpdateContentEntityContent,
) -> (DateTime<Utc>, String) {
    let uuid = content.id;
    let date: DateTime<Utc> = Utc::now();

    let log = format!(
        "{}|{}|{}|{}|{}|{}|{};",
        Action::UpdateContent,
        date.to_string(),
        uuid.to_string(),
        content.name,
        content.content_log,
        content.current_state,
        content.previous_registry
    );
    (date, log)
}

pub fn delete_entity_content(content: DeleteId) -> (DateTime<Utc>, String) {
    let date: DateTime<Utc> = Utc::now();

    let log = format!(
        "{}|{}|{}|{}|{}|{};",
        Action::Delete,
        date.to_string(),
        content.uuid.to_string(),
        content.name,
        content.content_log,
        content.previous_registry
    );
    (date, log)
}
