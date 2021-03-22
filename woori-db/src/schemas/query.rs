use std::collections::{BTreeMap, HashMap};

use crate::{core::pretty_config_output, model::error::Error, schemas::pretty_config};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wql::Types;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountResponse {
    response: String,
    count: usize,
}

impl CountResponse {
    pub fn new(count: usize, response: String) -> Self {
        Self { count, response }
    }

    // pub fn to_response(count: usize, response: String) -> Result<String, Error> {
    //     let resp = Self { count, response };
    //     Ok(ron::ser::to_string_pretty(&resp, pretty_config())?)
    // }
}

pub enum Response {
    Id(HashMap<String, Types>),
    All(BTreeMap<Uuid, HashMap<String, Types>>),
    Order(Vec<(Uuid, HashMap<String, Types>)>),
    GroupBy(HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>),
    OrderedGroupBy(HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>),
    OptionOrder(Vec<(Uuid, Option<HashMap<String, Types>>)>),
    OptionGroupBy(HashMap<String, BTreeMap<Uuid, Option<HashMap<String, Types>>>>),
    OptionSelect(BTreeMap<Uuid, Option<HashMap<String, Types>>>),
    CheckValues(HashMap<String, bool>),
    TimeRange(BTreeMap<DateTime<Utc>, HashMap<String, Types>>),
    WithCount(CountResponse),
    DateSelect(HashMap<String, HashMap<String, Types>>),
}

impl From<CountResponse> for Response {
    fn from(map: CountResponse) -> Self {
        Self::WithCount(map)
    }
}

impl From<HashMap<String, HashMap<String, Types>>> for Response {
    fn from(map: HashMap<String, HashMap<String, Types>>) -> Self {
        Self::DateSelect(map)
    }
}

impl From<BTreeMap<Uuid, Option<HashMap<String, Types>>>> for Response {
    fn from(map: BTreeMap<Uuid, Option<HashMap<String, Types>>>) -> Self {
        Self::OptionSelect(map)
    }
}

impl From<HashMap<String, Types>> for Response {
    fn from(map: HashMap<String, Types>) -> Self {
        Self::Id(map)
    }
}

impl From<HashMap<String, bool>> for Response {
    fn from(map: HashMap<String, bool>) -> Self {
        Self::CheckValues(map)
    }
}

impl From<BTreeMap<DateTime<Utc>, HashMap<String, Types>>> for Response {
    fn from(map: BTreeMap<DateTime<Utc>, HashMap<String, Types>>) -> Self {
        Self::TimeRange(map)
    }
}

impl From<BTreeMap<Uuid, HashMap<String, Types>>> for Response {
    fn from(map: BTreeMap<Uuid, HashMap<String, Types>>) -> Self {
        Self::All(map)
    }
}

impl From<Vec<(Uuid, HashMap<String, Types>)>> for Response {
    fn from(map: Vec<(Uuid, HashMap<String, Types>)>) -> Self {
        Self::Order(map)
    }
}

impl From<HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>> for Response {
    fn from(map: HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>) -> Self {
        Self::GroupBy(map)
    }
}

impl From<HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>> for Response {
    fn from(map: HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>) -> Self {
        Self::OrderedGroupBy(map)
    }
}

impl From<Vec<(Uuid, Option<HashMap<String, Types>>)>> for Response {
    fn from(map: Vec<(Uuid, Option<HashMap<String, Types>>)>) -> Self {
        Self::OptionOrder(map)
    }
}

impl From<HashMap<String, BTreeMap<Uuid, Option<HashMap<String, Types>>>>> for Response {
    fn from(map: HashMap<String, BTreeMap<Uuid, Option<HashMap<String, Types>>>>) -> Self {
        Self::OptionGroupBy(map)
    }
}

impl Response {
    pub fn to_string(self) -> Result<String, Error> {
        match self {
            Response::Id(state) => Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?),
            Response::CheckValues(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::TimeRange(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::All(state) => Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?),
            Response::Order(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::GroupBy(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::OrderedGroupBy(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::OptionOrder(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::OptionGroupBy(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::WithCount(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::OptionSelect(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::DateSelect(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
        }
    }
}
