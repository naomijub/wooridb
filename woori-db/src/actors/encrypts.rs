use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::Serialize;
// use wql::Types;

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
            let hm = msg
                .encrypts
                .into_iter()
                .map(|name| (name, HashSet::new()))
                .collect::<HashMap<String, HashSet<String>>>();
            encrypt_data.insert(msg.entity.to_owned(), hm);
        } else {
            msg.encrypts.iter().for_each(|name| {
                let mut hm = HashMap::new();
                hm.insert(name.to_owned(), HashSet::new());
                encrypt_data.entry(msg.entity.to_owned()).or_insert(hm);
            });
        }
        Ok(())
    }
}

// pub struct CheckForEncrypts {
//     pub entity: String,
//     pub content: HashMap<String, Types>,
//     pub encrypts: Arc<Arc<Mutex<EncryptContext>>>,
// }

// impl Message for CheckForEncrypts {
//     type Result = Result<(), Error>;
// }

// impl Handler<CheckForEncrypts> for Executor {
//     type Result = Result<(), Error>;

//     fn handle(&mut self, msg: CheckForEncrypts, _: &mut Self::Context) -> Self::Result {
//         let mut encrypts_data = if let Ok(guard) = msg.encrypts.lock() {
//             guard
//         } else {
//             return Err(Error::LockData);
//         };

//         if !encrypts_data.is_empty() {
//             let uniques_for_entity = encrypts_data
//                 .get_mut(&msg.entity)
//                 .ok_or_else(|| Error::EntityNotCreatedWithUniqueness(msg.entity.to_owned()))?;
//             msg.content.iter().try_for_each(|(k, v)| {
//                 if uniques_for_entity.contains_key(k) {
//                     let val = uniques_for_entity.get_mut(k).ok_or_else(|| {
//                         Error::EntityNotCreatedWithUniqueness(msg.entity.to_owned())
//                     })?;
//                     if val.contains(&format!("{:?}", v)) {
//                         Err(Error::DuplicatedUnique(
//                             msg.entity.to_owned(),
//                             k.to_owned(),
//                             v.to_owned(),
//                         ))
//                     } else {
//                         val.insert(format!("{:?}", v));
//                         Ok(())
//                     }
//                 } else {
//                     Ok(())
//                 }
//             })?;
//         }

//         Ok(())
//     }
// }

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
