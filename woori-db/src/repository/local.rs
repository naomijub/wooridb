use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::schemas::Role, model::DataRegister};

pub type LocalContext = BTreeMap<String, BTreeMap<Uuid, DataRegister>>;
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

    // pub fn is_valid_role(&self, role: Role) -> bool {
    //     self.roles.contains(&role)
    // }

    // pub fn is_valid_date(&self) -> bool {
    //     let now = Utc::now();
    //     self.expiration > now
    // }
}
