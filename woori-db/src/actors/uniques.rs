use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use ron::ser::to_string_pretty;
use serde::Serialize;
use wql::Types;

use crate::{
    actors::wql::Executor, core::pretty_config_inner, io::write::unique_data, model::error::Error,
    repository::local::UniquenessContext,
};

#[derive(Serialize)]
pub struct WriteWithUniqueKeys {
    pub entity: String,
    pub uniques: Vec<String>,
}

impl Message for WriteWithUniqueKeys {
    type Result = Result<(), Error>;
}

impl Handler<WriteWithUniqueKeys> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: WriteWithUniqueKeys, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_uniques;
        let unique_log =
            to_string_pretty(&msg, pretty_config_inner()).map_err(Error::Serialization)?;
        Ok(write_to_uniques(&unique_log)?)
    }
}

pub struct CreateWithUniqueKeys {
    pub entity: String,
    pub uniques: Vec<String>,
    pub data: Arc<Arc<Mutex<UniquenessContext>>>,
}

impl Message for CreateWithUniqueKeys {
    type Result = Result<(), Error>;
}

impl Handler<CreateWithUniqueKeys> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: CreateWithUniqueKeys, _: &mut Self::Context) -> Self::Result {
        let mut uniqueness_data = if let Ok(guard) = msg.data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };

        if uniqueness_data.contains_key(&msg.entity) {
            msg.uniques.iter().for_each(|name| {
                let mut hm = HashMap::new();
                hm.insert(name.to_owned(), HashSet::new());
                uniqueness_data.entry(msg.entity.to_owned()).or_insert(hm);
            });
        } else {
            let hm = msg
                .uniques
                .into_iter()
                .map(|name| (name, HashSet::new()))
                .collect::<HashMap<String, HashSet<String>>>();
            uniqueness_data.insert(msg.entity.to_owned(), hm);
        }
        let unique_ron =
            ron::ser::to_string_pretty(&uniqueness_data.clone(), pretty_config_inner())?;
        unique_data(&unique_ron)?;
        Ok(())
    }
}

pub struct CheckForUniqueKeys {
    pub entity: String,
    pub content: HashMap<String, Types>,
    pub uniqueness: Arc<Arc<Mutex<UniquenessContext>>>,
}

impl Message for CheckForUniqueKeys {
    type Result = Result<(), Error>;
}

impl Handler<CheckForUniqueKeys> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: CheckForUniqueKeys, _: &mut Self::Context) -> Self::Result {
        let mut uniqueness_data = if let Ok(guard) = msg.uniqueness.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };

        if !uniqueness_data.is_empty() {
            if let Some(uniques_for_entity) = uniqueness_data.get_mut(&msg.entity) {
                msg.content.iter().try_for_each(|(k, v)| {
                    if uniques_for_entity.contains_key(k) {
                        let val = uniques_for_entity.get_mut(k).ok_or_else(|| {
                            Error::EntityNotCreatedWithUniqueness(msg.entity.to_owned())
                        })?;
                        if val.contains(&format!("{:?}", v)) {
                            Err(Error::DuplicatedUnique(
                                msg.entity.to_owned(),
                                k.to_owned(),
                                v.to_owned(),
                            ))
                        } else {
                            val.insert(format!("{:?}", v));
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                })?;
            }
            let unique_ron =
                ron::ser::to_string_pretty(&uniqueness_data.clone(), pretty_config_inner())?;
            unique_data(&unique_ron)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::actors::wql::Executor;
    use crate::io::read::assert_uniques;

    #[actix_rt::test]
    async fn write_uniques() {
        let uniques = WriteWithUniqueKeys {
            entity: String::from("my-entity"),
            uniques: vec![String::from("id"), String::from("ssn")],
        };
        let actor = Executor::new().start();

        let resp = actor.send(uniques).await.unwrap();
        assert!(resp.is_ok());
        assert_uniques("uniques: [\"id\",\"ssn\",]");
    }

    #[actix_rt::test]
    async fn create_uniques_test() {
        let data = UniquenessContext::new();
        let uniques = CreateWithUniqueKeys {
            entity: String::from("my-entity"),
            uniques: vec![String::from("id"), String::from("ssn")],
            data: Arc::new(Arc::new(Mutex::new(data.clone()))),
        };
        let actor = Executor::new().start();

        let resp = actor.send(uniques).await.unwrap();
        assert!(resp.is_ok());
    }
}
