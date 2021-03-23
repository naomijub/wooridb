use crate::core::pretty_config_output;
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
        ron::ser::to_string_pretty(self, pretty_config_output())
            .unwrap_or_else(|_| "SERVER ERROR".to_string())
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
        ron::ser::to_string_pretty(self, pretty_config_output())
            .unwrap_or_else(|_| "SERVER ERROR".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteOrEvictEntityResponse {
    entity: String,
    uuid: Option<Uuid>,
    message: String,
}

impl DeleteOrEvictEntityResponse {
    pub fn new(entity: String, uuid: Option<Uuid>, message: String) -> Self {
        Self {
            entity,
            uuid,
            message,
        }
    }

    pub fn write(&self) -> String {
        ron::ser::to_string_pretty(self, pretty_config_output())
            .unwrap_or_else(|_| "SERVER ERROR".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntityResponse {
    entity: String,
    uuid: Uuid,
    state: String,
    message: String,
}

impl UpdateEntityResponse {
    pub fn new(entity: String, uuid: Uuid, state: String, message: String) -> Self {
        Self {
            entity,
            uuid,
            state,
            message,
        }
    }

    pub fn write(&self) -> String {
        ron::ser::to_string_pretty(self, pretty_config_output())
            .unwrap_or_else(|_| "SERVER ERROR".to_string())
    }
}
