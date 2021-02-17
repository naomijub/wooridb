use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
};

use crate::model::error::Error;
use bcrypt::{hash, DEFAULT_COST};
use chrono::{DateTime, Utc};
use ron::from_str;

use super::models::{AdminInfo, User, UserRegistry};
use super::schemas;

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
    file.write_all(log.as_bytes())?;

    Ok(())
}

pub async fn find_user(user: schemas::User) -> Result<UserRegistry, Error> {
    let users_info_log = "data/users_info.log";

    let file = OpenOptions::new().read(true).open(users_info_log)?;
    let buffer = BufReader::new(file);
    let uuid = user.id;

    let user_content = buffer
        .lines()
        .find(|l| (l.as_ref().unwrap_or(&String::new())).contains(&uuid.to_string()))
        .ok_or_else(|| Error::Unknown)??;

    let user: Result<UserRegistry, Error> = match from_str(&user_content) {
        Ok(u) => Ok(u),
        Err(_) => Err(Error::Unknown),
    };

    user
}

#[cfg(test)]
pub fn assert_users_content(pat: &str) {
    use chrono::prelude::*;
    use std::io::Read;

    let utc: DateTime<Utc> = Utc::now();
    let user_log = utc.format("data/users_info.log").to_string();

    let mut file = OpenOptions::new().read(true).open(user_log).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}
