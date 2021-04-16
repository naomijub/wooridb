#[cfg(not(feature = "json"))]
use crate::core::pretty_config_output;
use serde::{Deserialize, Serialize};
use wql::ID;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxType {
    Create,
    Insert,
    UpdateSet,
    UpdateContent,
    Delete,
    EvictEntity,
    EvictEntityTree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxResponse {
    tx_type: TxType,
    entity: String,
    pub(crate) uuid: Option<ID>,
    state: String,
    message: String,
}

impl TxResponse {
    pub fn write(&self) -> String {
        #[cfg(feature = "json")]
        return serde_json::to_string(self).unwrap_or_else(|_| "SERVER ERROR".to_string());
        #[cfg(not(feature = "json"))]
        ron::ser::to_string_pretty(self, pretty_config_output())
            .unwrap_or_else(|_| "SERVER ERROR".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityResponse {
    entity: String,
    message: String,
}

impl CreateEntityResponse {
    pub fn new(entity: String, message: String) -> Self {
        Self { entity, message }
    }
}

impl From<CreateEntityResponse> for TxResponse {
    fn from(tx: CreateEntityResponse) -> Self {
        Self {
            tx_type: TxType::Create,
            entity: tx.entity,
            uuid: None,
            state: String::new(),
            message: tx.message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertEntityResponse {
    entity: String,
    pub(crate) uuid: ID,
    message: String,
}

impl From<InsertEntityResponse> for TxResponse {
    fn from(tx: InsertEntityResponse) -> Self {
        Self {
            tx_type: TxType::Insert,
            entity: tx.entity,
            uuid: Some(tx.uuid),
            state: String::new(),
            message: tx.message,
        }
    }
}

impl InsertEntityResponse {
    pub fn new(entity: String, uuid: ID, message: String) -> Self {
        Self {
            entity,
            uuid,
            message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteOrEvictEntityResponse {
    entity: String,
    uuid: Option<ID>,
    message: String,
    tx_type: TxType,
}

impl From<DeleteOrEvictEntityResponse> for TxResponse {
    fn from(tx: DeleteOrEvictEntityResponse) -> Self {
        Self {
            tx_type: tx.tx_type,
            entity: tx.entity,
            uuid: tx.uuid,
            state: String::new(),
            message: tx.message,
        }
    }
}

impl DeleteOrEvictEntityResponse {
    pub fn new(entity: String, uuid: Option<ID>, message: String, tx_type: TxType) -> Self {
        Self {
            entity,
            uuid,
            message,
            tx_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntityResponse {
    entity: String,
    uuid: ID,
    state: String,
    message: String,
    tx_type: TxType,
}

impl From<UpdateEntityResponse> for TxResponse {
    fn from(tx: UpdateEntityResponse) -> Self {
        Self {
            tx_type: tx.tx_type,
            entity: tx.entity,
            uuid: Some(tx.uuid),
            state: tx.state,
            message: tx.message,
        }
    }
}

impl UpdateEntityResponse {
    pub fn new(entity: String, uuid: ID, state: String, message: String, tx_type: TxType) -> Self {
        Self {
            entity,
            uuid,
            state,
            message,
            tx_type,
        }
    }
}
