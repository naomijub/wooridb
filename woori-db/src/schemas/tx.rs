use crate::schemas::pretty_config;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct CreateEntityResponse {
    entity: String,
    message: String,
}

impl CreateEntityResponse {
    pub fn new(entity: String, message: String) -> Self {
        Self { entity, message }
    }

    pub fn write(&self) -> String {
        ron::ser::to_string_pretty(self, pretty_config()).unwrap_or("SERVER ERROR".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertEntityResponse {
    entity: String,
    pub(crate) uuid: Uuid,
    message: String,
}

impl InsertEntityResponse {
    pub fn new(entity: String, uuid: Uuid, message: String) -> Self {
        Self {
            entity,
            uuid,
            message,
        }
    }

    pub fn write(&self) -> String {
        ron::ser::to_string_pretty(self, pretty_config()).unwrap_or("SERVER ERROR".to_string())
    }
}
