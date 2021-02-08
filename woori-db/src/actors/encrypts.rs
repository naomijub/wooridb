use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::Serialize;
use wql::Types;

use crate::{actors::wql::Executor, model::error::Error, repository::local::EncryptContext};

#[derive(Serialize)]
pub struct WriteEncrypts {
    pub entity: String,
    pub encrypts: Vec<String>,
}

impl Message for WriteEncrypts {
    type Result = Result<(), Error>;
}

impl Handler<WriteEncrypts> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: WriteEncrypts, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_encrypts;
        let encrypt_log =
            to_string_pretty(&msg, pretty_config()).map_err(Error::SerializationError)?;
        Ok(write_to_encrypts(&encrypt_log)?)
    }
}

pub struct CreateEncrypts {
    pub entity: String,
    pub encrypts: Vec<String>,
    pub data: Arc<Arc<Mutex<EncryptContext>>>,
}

impl Message for CreateEncrypts {
    type Result = Result<(), Error>;
}

impl Handler<CreateEncrypts> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: CreateEncrypts, _: &mut Self::Context) -> Self::Result {
        let mut encrypt_data = if let Ok(guard) = msg.data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };

        if !encrypt_data.contains_key(&msg.entity) {
            let hm = msg.encrypts.into_iter().collect::<HashSet<String>>();
            encrypt_data.insert(msg.entity.to_owned(), hm);
        }
        Ok(())
    }
}

pub struct EncryptContent {
    pub entity: String,
    pub content: HashMap<String, Types>,
    pub encrypts: Arc<Arc<Mutex<EncryptContext>>>,
}

impl EncryptContent {
    pub fn new(
        entity: &str,
        content: HashMap<String, Types>,
        encrypts: Arc<Arc<Mutex<EncryptContext>>>,
    ) -> Self {
        Self {
            entity: entity.to_owned(),
            content,
            encrypts,
        }
    }
}

impl Message for EncryptContent {
    type Result = Result<HashMap<String, Types>, Error>;
}

impl Handler<EncryptContent> for Executor {
    type Result = Result<HashMap<String, Types>, Error>;

    fn handle(&mut self, msg: EncryptContent, _: &mut Self::Context) -> Self::Result {
        let mut encrypts_data = if let Ok(guard) = msg.encrypts.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };

        if !encrypts_data.is_empty() {
            if let Some(encrypts_for_entity) = encrypts_data.get_mut(&msg.entity) {
                let mut new_content = HashMap::new();
                msg.content.iter().for_each(|(k, v)| {
                    if encrypts_for_entity.contains(k) {
                        // fix unwrap
                        // Maybe https://docs.rs/bcrypt/0.9.0/bcrypt/fn.hash_with_salt.html
                        let hashed_v = v.to_hash(Some(14u32)).unwrap();
                        new_content.insert(k.to_owned(), hashed_v);
                    } else {
                        new_content.insert(k.to_owned(), v.to_owned());
                    }
                });

                Ok(new_content)
            } else {
                Ok(msg.content)
            }
        } else {
            Ok(msg.content)
        }
    }
}

fn pretty_config() -> PrettyConfig {
    PrettyConfig::new()
        .with_indentor("".to_string())
        .with_new_line("".to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{actors::wql::Executor, io::read::assert_encrypt};

    #[actix_rt::test]
    async fn write_uniques() {
        let encrypts = WriteEncrypts {
            entity: String::from("my-entity"),
            encrypts: vec![String::from("id"), String::from("ssn")],
        };
        let actor = Executor::new().start();

        let resp = actor.send(encrypts).await.unwrap();
        assert!(resp.is_ok());
        assert_encrypt("encrypts: [\"id\",\"ssn\",]");
    }

    #[actix_rt::test]
    async fn create_uniques_test() {
        let data = EncryptContext::new();
        let encrypts = CreateEncrypts {
            entity: String::from("my-entity"),
            encrypts: vec![String::from("id"), String::from("ssn")],
            data: Arc::new(Arc::new(Mutex::new(data.clone()))),
        };
        let actor = Executor::new().start();

        let resp = actor.send(encrypts).await.unwrap();
        assert!(resp.is_ok());
    }
}
