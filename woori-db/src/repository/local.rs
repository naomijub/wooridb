use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wql::Types;

use crate::{auth::schemas::Role, model::DataRegister};

pub type LocalContext = BTreeMap<String, BTreeMap<Uuid, (DataRegister, HashMap<String, Types>)>>;
pub type UniquenessContext = BTreeMap<String, HashMap<String, HashSet<String>>>;
pub type EncryptContext = BTreeMap<String, HashSet<String>>;
pub type SessionContext = BTreeMap<String, SessionInfo>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionInfo {
    expiration: DateTime<Utc>,
    roles: Vec<Role>,
}

impl SessionInfo {
    pub fn new(expiration: DateTime<Utc>, roles: Vec<Role>) -> Self {
        Self { expiration, roles }
    }

    #[cfg(not(debug_assertions))]
    pub fn is_valid_role(&self, roles: Vec<Role>) -> bool {
        roles.iter().any(|role| self.roles.contains(&role))
    }

    #[cfg(not(debug_assertions))]
    pub fn is_valid_date(&self) -> bool {
        let now = Utc::now();
        self.expiration > now
    }
}
