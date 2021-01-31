use actix::prelude::*;
use std::collections::HashMap;
use wql::Types;

use crate::model::error::Error;
use crate::{actors::wql::Executor, model::DataRegister};

pub struct State(pub String);

impl Message for State {
    type Result = Result<HashMap<String, Types>, Error>;
}

impl Handler<State> for Executor {
    type Result = Result<HashMap<String, Types>, Error>;

    fn handle(&mut self, msg: State, _: &mut Self::Context) -> Self::Result {
        use ron::de::from_str;

        let fractions = msg.0.split("|").collect::<Vec<&str>>();
        if fractions[0].eq("INSERT") {
            let state = fractions.last().unwrap().to_owned();
            let state = &state[..(state.len() - 1)];

            let resp: Result<HashMap<String, Types>, Error> = match from_str(state) {
                Ok(x) => Ok(x),
                Err(_) => Err(Error::FailedToParseState),
            };
            resp
        } else if fractions[0].eq("UPDATE_SET") || fractions[0].eq("UPDATE_CONTENT") {
            let state = fractions[fractions.len() - 2];

            let resp: Result<HashMap<String, Types>, Error> = match from_str(state) {
                Ok(x) => Ok(x),
                Err(_) => Err(Error::FailedToParseState),
            };
            resp
        } else {
            Err(Error::FailedToParseState)
        }
    }
}

pub struct PreviousRegistry(pub String);

impl Message for PreviousRegistry {
    type Result = Result<Option<DataRegister>, Error>;
}

impl Handler<PreviousRegistry> for Executor {
    type Result = Result<Option<DataRegister>, Error>;

    fn handle(&mut self, msg: PreviousRegistry, _: &mut Self::Context) -> Self::Result {
        use ron::de::from_str;

        let fractions = msg.0.split("|").collect::<Vec<&str>>();
        if fractions[0].eq("INSERT") {
            Ok(None)
        } else if fractions[0].eq("UPDATE_SET") || fractions[0].eq("UPDATE_CONTENT") {
            let state = fractions.last().unwrap().to_owned();
            let state = &state[..(state.len() - 1)];

            let resp: Result<DataRegister, Error> = match from_str(state) {
                Ok(x) => Ok(x),
                Err(_) => Err(Error::FailedToParseRegistry),
            };
            Ok(Some(resp?))
        } else {
            Err(Error::FailedToParseRegistry)
        }
    }
}
