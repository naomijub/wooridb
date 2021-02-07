use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::Serialize;
use wql::Types;

use crate::{actors::wql::Executor, model::error::Error, repository::local::UniquenessContext};

#[derive(Serialize)]
pub struct WriteUniques {
    pub entity: String,
    pub uniques: Vec<String>,
}

impl Message for WriteUniques {
    type Result = Result<(), Error>;
}

impl Handler<WriteUniques> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: WriteUniques, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_uniques;
        let unique_log =
            to_string_pretty(&msg, pretty_config()).map_err(Error::SerializationError)?;
        Ok(write_to_uniques(&unique_log)?)
    }
}

pub struct CreateUniques {
    pub entity: String,
    pub uniques: Vec<String>,
    pub data: Arc<Arc<Mutex<UniquenessContext>>>,
}

impl Message for CreateUniques {
    type Result = Result<(), Error>;
}

impl Handler<CreateUniques> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: CreateUniques, _: &mut Self::Context) -> Self::Result {
        let mut uniqueness_data = if let Ok(guard) = msg.data.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };

        if !uniqueness_data.contains_key(&msg.entity) {
            let hm = msg
                .uniques
                .into_iter()
                .map(|name| (name, HashSet::new()))
                .collect::<HashMap<String, HashSet<String>>>();
            uniqueness_data.insert(msg.entity.to_string(), hm);
        } else {
            msg.uniques.iter().for_each(|name| {
                let mut hm = HashMap::new();
                hm.insert(name.to_owned(), HashSet::new());
                uniqueness_data.entry(msg.entity.to_owned()).or_insert(hm);
            });
        }
        Ok(())
    }
}

pub struct CheckForUnique {
    pub entity: String,
    pub content: HashMap<String, Types>,
    pub uniqueness: Arc<Arc<Mutex<UniquenessContext>>>,
}

impl Message for CheckForUnique {
    type Result = Result<(), Error>;
}

impl Handler<CheckForUnique> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: CheckForUnique, _: &mut Self::Context) -> Self::Result {
        let mut uniqueness_data = if let Ok(guard) = msg.uniqueness.lock() {
            guard
        } else {
            return Err(Error::LockData);
        };

        if !uniqueness_data.is_empty() {
            let uniques_for_entity = uniqueness_data
                .get_mut(&msg.entity)
                .ok_or_else(|| Error::EntityNotCreatedWithUniqueness(msg.entity.to_owned()))?;
            msg.content.iter().try_for_each(|(k, v)| {
                if uniques_for_entity.contains_key(k) {
                    let val = uniques_for_entity
                        .get_mut(k)
                        .ok_or_else(|| Error::EntityNotCreatedWithUniqueness(msg.entity.to_owned()))?;
                    if val.contains(&format!("{:?}", v)) {
                        Err(Error::DuplicatedUnique(
                            msg.entity.to_owned(),
                            k.to_string(),
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

        Ok(())
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
    use crate::actors::wql::Executor;
    use crate::io::read::assert_uniques;

    #[actix_rt::test]
    async fn write_uniques() {
        let uniques = WriteUniques {
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
        let uniques = CreateUniques {
            entity: String::from("my-entity"),
            uniques: vec![String::from("id"), String::from("ssn")],
            data: Arc::new(Arc::new(Mutex::new(data.clone()))),
        };
        let actor = Executor::new().start();

        let resp = actor.send(uniques).await.unwrap();
        assert!(resp.is_ok());
    }
}
