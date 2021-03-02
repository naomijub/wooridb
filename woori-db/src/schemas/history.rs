use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct EntityHistoryInfo {
    pub entity_key: String,
    pub entity_id: Uuid,
}
