use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::Serialize;

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
            to_string_pretty(&msg, pretty_config()).map_err(|e| Error::SerializationError(e))?;
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
        let mut uniqueness_data = msg.data.lock().unwrap();
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
                hm.insert(name.clone(), HashSet::new());
                uniqueness_data.entry(msg.entity.clone()).or_insert(hm);
            });
        }
        Ok(())
    }
}

fn pretty_config() -> PrettyConfig {
    PrettyConfig::new()
        .with_indentor("".to_string())
        .with_new_line("".to_string())
}
