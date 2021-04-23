use actix::prelude::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use wql::Types;

use crate::model::error::Error;
use crate::{actors::wql::Executor, model::DataRegister};

pub type HistoryRegistry = (HashMap<String, Types>, DateTime<Utc>, Option<DataRegister>);
pub struct History(pub String);

impl Message for History {
    type Result = Result<HistoryRegistry, Error>;
}

impl Handler<History> for Executor {
    type Result = Result<HistoryRegistry, Error>;

    fn handle(&mut self, msg: History, _: &mut Self::Context) -> Self::Result {
        let fractions = msg.0.split('|').collect::<Vec<&str>>();
        if fractions[0].eq("INSERT") {
            let date = get_date(&fractions);
            let content = get_insert_content(&fractions);
            let previous_registry = None;
            Ok((content?, date?, previous_registry))
        } else if fractions[0].eq("UPDATE_SET")
            || fractions[0].eq("UPDATE_CONTENT")
            || fractions[0].eq("DELETE")
        {
            let date = get_date(&fractions);
            let content = get_other_content(&fractions);
            let previous_registry = get_previous_registry(&fractions);
            Ok((content?, date?, previous_registry?))
        } else {
            Err(Error::FailedToParseState)
        }
    }
}

fn get_insert_content(fractions: &[&str]) -> Result<HashMap<String, Types>, Error> {
    let state = fractions
        .last()
        .ok_or(Error::FailedToParseState)?
        .to_owned();
    let state = &state[..(state.len() - 1)];

    let resp: Result<HashMap<String, Types>, Error> = match ron::de::from_str(state) {
        Ok(x) => Ok(x),
        Err(_) => Err(Error::FailedToParseState),
    };
    resp
}

fn get_other_content(fractions: &[&str]) -> Result<HashMap<String, Types>, Error> {
    let state = fractions
        .get(fractions.len() - 2)
        .ok_or(Error::FailedToParseState)?
        .to_owned();

    let resp: Result<HashMap<String, Types>, Error> = match ron::de::from_str(state) {
        Ok(x) => Ok(x),
        Err(_) => Err(Error::FailedToParseState),
    };
    resp
}

fn get_date(fractions: &[&str]) -> Result<DateTime<Utc>, Error> {
    let state = fractions
        .get(1)
        .ok_or(Error::FailedToParseState)?
        .to_owned();

    let resp: Result<DateTime<Utc>, Error> = match ron::de::from_str(state) {
        Ok(x) => Ok(x),
        Err(_) => Err(Error::FailedToParseState),
    };
    resp
}

fn get_previous_registry(fractions: &[&str]) -> Result<Option<DataRegister>, Error> {
    let state = fractions
        .last()
        .ok_or(Error::FailedToParseRegistry)?
        .to_owned();
    let state = &state[..(state.len() - 1)];

    let resp: Result<(DataRegister, Vec<u8>), Error> = match ron::de::from_str(state) {
        Ok(x) => Ok(x),
        Err(_) => Err(Error::FailedToParseRegistry),
    };
    let resp_insert: Result<DataRegister, Error> = match ron::de::from_str(state) {
        Ok(x) => Ok(x),
        Err(_) => Err(Error::FailedToParseRegistry),
    };
    if resp.is_ok() {
        Ok(Some(resp?.0))
    } else {
        Ok(Some(resp_insert?))
    }
}
