use actix::prelude::*;
use std::collections::HashMap;
use wql::Types;

use crate::{io::read::read_date_log, model::error::Error};

use super::wql::Executor;

pub struct ReadEntitiesAt {
    entity_name: String,
    date_log: String,
}

impl ReadEntitiesAt {
    pub fn new(entity_name: &str, date_log: String) -> Self {
        Self {
            entity_name: entity_name.to_owned(),
            date_log,
        }
    }
}

impl Message for ReadEntitiesAt {
    type Result = Result<HashMap<String, HashMap<String, Types>>, Error>;
}

impl Handler<ReadEntitiesAt> for Executor {
    type Result = Result<HashMap<String, HashMap<String, Types>>, Error>;

    fn handle(&mut self, msg: ReadEntitiesAt, _: &mut Self::Context) -> Self::Result {
        use ron::de::from_str;
        let date_log = read_date_log(msg.date_log)?;
        let mut hm = HashMap::new();
        date_log.split(";").into_iter().try_for_each(|line| {
            let fractions = line.split('|').collect::<Vec<&str>>();
            if fractions[3].eq(&msg.entity_name) {
                if fractions[0].eq("INSERT") {
                    let state = fractions
                        .last()
                        .ok_or_else(|| Error::FailedToParseState)?
                        .to_owned();

                    let resp: Result<HashMap<String, Types>, Error> = match from_str(state) {
                        Ok(x) => Ok(x),
                        Err(_) => Err(Error::FailedToParseState),
                    };
                    match resp {
                        Ok(map) => {
                            hm.insert(fractions[2].to_owned(), map);
                        }
                        Err(e) => return Err(e),
                    };
                }
            } else if fractions[0].eq("UPDATE_SET") || fractions[0].eq("UPDATE_CONTENT") {
                let state = fractions
                    .get(fractions.len() - 2)
                    .ok_or_else(|| Error::FailedToParseState)?
                    .to_owned();

                let resp: Result<HashMap<String, Types>, Error> = match from_str(state) {
                    Ok(x) => Ok(x),
                    Err(_) => Err(Error::FailedToParseState),
                };
                match resp {
                    Ok(map) => {
                        hm.insert(fractions[2].to_owned(), map);
                    }
                    Err(e) => return Err(e),
                };
            }
            Ok(())
        });

        Ok(hm)
    }
}
