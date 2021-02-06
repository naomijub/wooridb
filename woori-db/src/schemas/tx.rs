use crate::schemas::pretty_config;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct EntityResponse {
    entity: String,
    message: String,
}

impl EntityResponse {
    pub fn new(entity: String, message: String) -> Self {
        Self { entity, message }
    }

    pub fn write(&self) -> String {
        ron::ser::to_string_pretty(self, pretty_config()).unwrap_or("SERVER ERROR".to_string())
    }
}
