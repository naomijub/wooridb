use chrono::{DateTime, Utc};
use serde::Deserialize;
use wql::ID;

#[derive(Debug, Clone, Deserialize)]
pub struct EntityHistoryInfo {
    pub entity_key: String,
    pub entity_id: ID,
    pub start_datetime: Option<DateTime<Utc>>,
    pub end_datetime: Option<DateTime<Utc>>,
}
