use bcrypt::verify;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{core::pretty_config_inner, model::error::Error};

use super::schemas::Role;

#[derive(Serialize, Deserialize)]
pub struct AdminInfo {
    admin_id: String,
    admin_hash: String,
    cost: u32,
}

impl AdminInfo {
    pub fn new(id: String, hash: String, cost: u32) -> Self {
        Self {
            admin_id: id,
            admin_hash: hash,
            cost,
        }
    }

    pub fn is_valid_hash(&self, pswd: &str, id: &str) -> bool {
        match verify(pswd, &self.admin_hash) {
            Ok(b) => b && id == self.admin_id,
            Err(_) => false,
        }
    }

    pub fn cost(&self) -> u32 {
        self.cost
    }
}

#[derive(Serialize, Deserialize)]
pub struct User {
    id: Uuid,
    user_password: String,
    roles: Vec<Role>,
}

#[derive(Serialize, Deserialize)]
pub struct UserRegistry {
    id: Uuid,
    hash: String,
    roles: Vec<Role>,
    date: DateTime<Utc>,
}

impl UserRegistry {
    pub fn context(self) -> (String, Vec<Role>) {
        (self.hash, self.roles)
    }
}

impl User {
    pub fn new(id: Uuid, user_password: String, roles: Vec<Role>) -> Self {
        Self {
            id,
            user_password,
            roles,
        }
    }

    pub fn format_user_log(&self, date: DateTime<Utc>) -> Result<String, Error> {
        let value = UserRegistry {
            id: self.id,
            hash: self.user_password.clone(),
            roles: self.roles.clone(),
            date,
        };
        Ok(format!(
            "{}\r\n",
            ron::ser::to_string_pretty(&value, pretty_config_inner())?
        ))
    }
}
