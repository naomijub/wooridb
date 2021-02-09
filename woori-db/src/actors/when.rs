use actix::prelude::*;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;
use wql::Types;

use crate::{io::read::read_date_log, model::error::Error};

use super::wql::Executor;
pub struct ReadEntityRange {
    entity_name: String,
    uuid: Uuid,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    date_log: String,
}

impl ReadEntityRange {
    pub fn new(
        entity_name: &str,
        uuid: Uuid,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        date_log: String,
    ) -> Self {
        Self {
            entity_name: entity_name.to_owned(),
            uuid,
            start_date,
            end_date,
            date_log,
        }
    }
}

impl Message for ReadEntityRange {
    type Result = Result<BTreeMap<DateTime<Utc>, HashMap<String, Types>>, Error>;
}

impl Handler<ReadEntityRange> for Executor {
    type Result = Result<BTreeMap<DateTime<Utc>, HashMap<String, Types>>, Error>;

    fn handle(&mut self, msg: ReadEntityRange, _: &mut Self::Context) -> Self::Result {
        use ron::de::from_str;
        let date_log = msg.date_log.clone();
        let date_log = read_date_log(date_log)?;
        let mut hm = BTreeMap::new();
        date_log.split(';').try_for_each(|line| {
            let fractions = line.split('|').collect::<Vec<&str>>();

            if fractions[0].eq("INSERT")
                && fractions[3].eq(&msg.entity_name)
                && fractions[2].eq(&msg.uuid.to_string())
            {
                let state = fractions
                    .last()
                    .ok_or_else(|| Error::FailedToParseState)?
                    .to_owned();
                let date: Result<DateTime<Utc>, Error> = match from_str(fractions[1]) {
                    Ok(x) => Ok(x),
                    Err(_) => Err(Error::FailedToParseDate),
                };
                let date = date?;

                if date > msg.start_date && date < msg.end_date {
                    let resp: Result<HashMap<String, Types>, Error> = match from_str(state) {
                        Ok(x) => Ok(x),
                        Err(_) => Err(Error::FailedToParseState),
                    };
                    match resp {
                        Ok(map) => {
                            let map = map.into_iter().filter(|(_, v)| !v.is_hash()).collect();
                            hm.insert(date, map);
                        }
                        Err(e) => return Err(e),
                    };
                }
            } else if (fractions[0].eq("UPDATE_SET") || fractions[0].eq("UPDATE_CONTENT"))
                && fractions[3].eq(&msg.entity_name)
                && fractions[2].eq(&msg.uuid.to_string())
            {
                let state = fractions
                    .get(fractions.len() - 2)
                    .ok_or_else(|| Error::FailedToParseState)?
                    .to_owned();
                let date: Result<DateTime<Utc>, Error> = match from_str(fractions[1]) {
                    Ok(x) => Ok(x),
                    Err(_) => Err(Error::FailedToParseDate),
                };
                let date = date?;

                if date > msg.start_date && date < msg.end_date {
                    let resp: Result<HashMap<String, Types>, Error> = match from_str(state) {
                        Ok(x) => Ok(x),
                        Err(_) => Err(Error::FailedToParseState),
                    };
                    match resp {
                        Ok(map) => {
                            let map = map.into_iter().filter(|(_, v)| !v.is_hash()).collect();
                            hm.insert(date, map);
                        }
                        Err(e) => return Err(e),
                    };
                }
            }
            Ok(())
        })?;

        Ok(hm)
    }
}

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
        let date_log = msg.date_log.clone();
        let date_log = read_date_log(date_log)?;
        let mut hm = HashMap::new();
        date_log.split(';').try_for_each(|line| {
            let fractions = line.split('|').collect::<Vec<&str>>();
            if fractions[0].eq("INSERT") && fractions[3].eq(&msg.entity_name) {
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
                        let map = map.into_iter().filter(|(_, v)| !v.is_hash()).collect();
                        hm.insert(fractions[2].to_owned(), map);
                    }
                    Err(e) => return Err(e),
                };
            } else if (fractions[0].eq("UPDATE_SET") || fractions[0].eq("UPDATE_CONTENT"))
                && fractions[3].eq(&msg.entity_name)
            {
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
                        let map = map.into_iter().filter(|(_, v)| !v.is_hash()).collect();
                        hm.insert(fractions[2].to_owned(), map);
                    }
                    Err(e) => return Err(e),
                };
            }
            Ok(())
        })?;

        Ok(hm)
    }
}

pub struct ReadEntityIdAt {
    entity_name: String,
    uuid: Uuid,
    date_log: String,
}

impl ReadEntityIdAt {
    pub fn new(entity_name: &str, uuid: Uuid, date_log: String) -> Self {
        Self {
            entity_name: entity_name.to_owned(),
            uuid,
            date_log,
        }
    }
}

impl Message for ReadEntityIdAt {
    type Result = Result<HashMap<String, Types>, Error>;
}

impl Handler<ReadEntityIdAt> for Executor {
    type Result = Result<HashMap<String, Types>, Error>;

    fn handle(&mut self, msg: ReadEntityIdAt, _: &mut Self::Context) -> Self::Result {
        use ron::de::from_str;
        let date_log = msg.date_log.clone();
        let date_log = read_date_log(date_log)?;
        let mut hm = HashMap::new();
        date_log.split(';').try_for_each(|line| {
            let fractions = line.split('|').collect::<Vec<&str>>();

            if fractions[0].eq("INSERT")
                && fractions[3].eq(&msg.entity_name)
                && fractions[2].eq(&msg.uuid.to_string())
            {
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
                        let map = map.into_iter().filter(|(_, v)| !v.is_hash()).collect();
                        hm = map;
                    }
                    Err(e) => return Err(e),
                };
            } else if (fractions[0].eq("UPDATE_SET") || fractions[0].eq("UPDATE_CONTENT"))
                && fractions[3].eq(&msg.entity_name)
                && fractions[2].eq(&msg.uuid.to_string())
            {
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
                        let map = map.into_iter().filter(|(_, v)| !v.is_hash()).collect();
                        hm = map;
                    }
                    Err(e) => return Err(e),
                };
            }
            Ok(())
        })?;

        Ok(hm)
    }
}
