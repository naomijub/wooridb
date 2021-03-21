use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use wql::Types;

use crate::{actors::wql::Executor, model::error::Error, repository::local::EncryptContext};

#[derive(Debug, Serialize, Deserialize)]
pub struct WriteWithEncryption {
    pub entity: String,
    pub encrypts: Vec<String>,
}

impl Message for WriteWithEncryption {
    type Result = Result<(), Error>;
}

impl Handler<WriteWithEncryption> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: WriteWithEncryption, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_encrypts;
        let encrypt_log = to_string_pretty(&msg, pretty_config()).map_err(Error::Serialization)?;
        Ok(write_to_encrypts(&encrypt_log)?)
    }
}

pub struct CreateWithEncryption {
    pub entity: String,
    pub encrypts: Vec<String>,
    pub data: Arc<Arc<Mutex<EncryptContext>>>,
}

impl Message for CreateWithEncryption {
    type Result = Result<(), Error>;
}

impl Handler<CreateWithEncryption> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: CreateWithEncryption, _: &mut Self::Context) -> Self::Result {
        let mut encrypt_data = if let Ok(guard) = msg.data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };

        if !encrypt_data.contains_key(&msg.entity) {
            let hm = msg.encrypts.into_par_iter().collect::<HashSet<String>>();
            encrypt_data.insert(msg.entity.to_owned(), hm);
        }
        Ok(())
    }
}

pub struct EncryptContent {
    pub entity: String,
    pub content: HashMap<String, Types>,
    pub encrypts: Arc<Arc<Mutex<EncryptContext>>>,
    pub hashing_cost: u32,
}

impl EncryptContent {
    pub fn new(
        entity: &str,
        mut content: HashMap<String, Types>,
        encrypts: Arc<Arc<Mutex<EncryptContext>>>,
        hashing_cost: u32,
        datetime: DateTime<Utc>,
    ) -> Self {
        content.insert("tx_time".to_owned(), Types::DateTime(datetime));
        Self {
            entity: entity.to_owned(),
            content,
            encrypts,
            hashing_cost,
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

        if encrypts_data.is_empty() {
            Ok(msg.content)
        } else {
            encrypts_data.get_mut(&msg.entity).map_or(
                Ok(msg.content.clone()),
                |encrypts_for_entity| {
                    let mut new_content = HashMap::new();
                    msg.content.iter().for_each(|(k, v)| {
                        if encrypts_for_entity.contains(k) {
                            #[cfg(test)]
                            let hashed_v = v.to_hash(Some(4)).unwrap();
                            #[cfg(not(test))]
                            let hashed_v = v.to_hash(Some(msg.hashing_cost)).unwrap();
                            new_content.insert(k.to_owned(), hashed_v);
                        } else {
                            new_content.insert(k.to_owned(), v.to_owned());
                        }
                    });

                    Ok(new_content)
                },
            )
        }
    }
}

pub struct VerifyEncryption {
    filtered: HashMap<String, Types>,
    content: HashMap<String, String>,
}

impl VerifyEncryption {
    pub fn new(filtered: HashMap<String, Types>, content: HashMap<String, String>) -> Self {
        Self { filtered, content }
    }
}

impl Message for VerifyEncryption {
    type Result = Result<String, Error>;
}

impl Handler<VerifyEncryption> for Executor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: VerifyEncryption, _: &mut Self::Context) -> Self::Result {
        let type_nil = Types::Nil;
        let results = msg
            .content
            .clone()
            .into_par_iter()
            .map(|(k, v)| {
                let original = msg.filtered.clone();
                let original_hash = original.get(&k).unwrap_or(&type_nil);
                let result = if let Types::Hash(hash) = original_hash {
                    bcrypt::verify(v, hash).unwrap()
                } else {
                    false
                };
                (k, result)
            })
            .collect::<HashMap<String, bool>>();
        let encrypt_log =
            to_string_pretty(&results, pretty_config()).map_err(Error::Serialization)?;

        Ok(encrypt_log)
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
        let encrypts = WriteWithEncryption {
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
        let encrypts = CreateWithEncryption {
            entity: String::from("my-entity"),
            encrypts: vec![String::from("id"), String::from("ssn")],
            data: Arc::new(Arc::new(Mutex::new(data.clone()))),
        };
        let actor = Executor::new().start();

        let resp = actor.send(encrypts).await.unwrap();
        assert!(resp.is_ok());
    }
}
