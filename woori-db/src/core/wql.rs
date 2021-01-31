use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{actors::wql::InsertEntityContent, model::wql::Action};

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
