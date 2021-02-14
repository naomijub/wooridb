use std::{
    fs::OpenOptions,
    io::{Read, Write},
};

use crate::model::error::Error;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};

use super::models::{AdminInfo, User};

pub fn read_admin_info() -> Result<AdminInfo, Error> {
    #[cfg(test)]
    let admin = std::env::var("ADMIN").unwrap_or("your_admin".to_string());
    #[cfg(not(test))]
    let admin = std::env::var("ADMIN").map_err(|_| Error::AdminNotConfigured)?;
    #[cfg(test)]
    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or("your_password".to_string());
    #[cfg(not(test))]
    let admin_password = std::env::var("ADMIN_PASSWORD").map_err(|_| Error::AdminNotConfigured)?;
    #[cfg(test)]
    let auth_hashing_cost = std::env::var("AUTH_HASHING_COST").unwrap_or("4".to_string());
    #[cfg(not(test))]
    let auth_hashing_cost =
        std::env::var("AUTH_HASHING_COST").map_err(|_| Error::AdminNotConfigured)?;
    let cost = auth_hashing_cost.parse::<u32>().unwrap_or(DEFAULT_COST);

    let pswd_hash = match hash(&admin_password, cost) {
        Ok(hash) => hash,
        Err(_) => return Err(Error::AdminNotConfigured),
    };

    Ok(AdminInfo::new(admin, pswd_hash, cost))
}

pub fn to_users_log(user: &User) -> Result<(), Error> {
    let utc: DateTime<Utc> = Utc::now();
    let users_info_log = "data/users_info.log";

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(users_info_log)?;

    let log = user.format_user_log(utc)?;
    file.write(log.as_bytes())?;

    Ok(())
}

#[cfg(test)]
pub fn assert_users_content(pat: &str) {
    use chrono::prelude::*;
    let utc: DateTime<Utc> = Utc::now();
    let date_log = utc.format("data/users_info.log").to_string();

    let mut file = OpenOptions::new().read(true).open(date_log).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}
