use actix::prelude::*;
use std::collections::HashMap;
use wql::Types;

use crate::actors::wql::Executor;
use crate::model::error::Error;

pub struct State(pub String);

impl Message for State {
    type Result = Result<HashMap<String, Types>, Error>;
}

impl Handler<State> for Executor {
    type Result = Result<HashMap<String, Types>, Error>;

    fn handle(&mut self, msg: State, _: &mut Self::Context) -> Self::Result {
        use ron::de::from_str;

        let fractions = msg.0.split("|").collect::<Vec<&str>>();
        #[cfg(test)] 
        {
            let vecs: Vec<&str> = Vec::new();
            assert_eq!(vecs, fractions);
        }
        if fractions[0].eq("INSERT") {
            let state = fractions.last().unwrap().to_owned();
            let state = &state[..(state.len() - 1)];

            #[cfg(test)] {
                assert_eq!("state", state);
            }
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
