use std::collections::{BTreeMap, HashMap};

use crate::{core::pretty_config_output, model::error::Error};
use chrono::{DateTime, Utc};
use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wql::Types;
#[cfg(feature = "json_pure")]
use wql::TypesSelfDescribing;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountResponse {
    response: Box<Response>,
    count: usize,
}

#[derive(Serialize)]
pub struct CountId {
    response: HashMap<String, Types>,
    count: usize,
}
#[derive(Serialize)]
pub struct CountAll {
    #[cfg(feature = "json_pure")]
    response: BTreeMap<Uuid, HashMap<String, TypesSelfDescribing>>,
    #[cfg(not(feature = "json_pure"))]
    response: BTreeMap<Uuid, HashMap<String, Types>>,
    count: usize,
}
#[derive(Serialize)]
pub struct CountOrder {
    response: Vec<(Uuid, HashMap<String, Types>)>,
    count: usize,
}
#[derive(Serialize)]
pub struct CountGroupBy {
    response: HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>,
    count: usize,
}
#[derive(Serialize)]
pub struct CountOrderedGroupBy {
    response: HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>,
    count: usize,
}
#[derive(Serialize)]
pub struct CountOptionOrder {
    response: Vec<(Uuid, Option<HashMap<String, Types>>)>,
    count: usize,
}
#[derive(Serialize)]
pub struct CountOptionGroupBy {
    response: HashMap<String, BTreeMap<Uuid, Option<HashMap<String, Types>>>>,
    count: usize,
}
#[derive(Serialize)]
pub struct CountOptionSelect {
    #[cfg(feature = "json_pure")]
    response: BTreeMap<Uuid, Option<HashMap<String, TypesSelfDescribing>>>,
    #[cfg(not(feature = "json_pure"))]
    response: BTreeMap<Uuid, Option<HashMap<String, Types>>>,
    count: usize,
}
#[derive(Serialize)]
pub struct CountCheckValues {
    response: HashMap<String, bool>,
    count: usize,
}
#[derive(Serialize)]
pub struct CountTimeRange {
    response: BTreeMap<DateTime<Utc>, HashMap<String, Types>>,
    count: usize,
}

#[derive(Serialize)]
pub struct CountDateSelect {
    response: HashMap<String, HashMap<String, Types>>,
    count: usize,
}

impl CountResponse {
    pub fn new(count: usize, response: Response) -> Self {
        Self {
            count,
            response: Box::new(response),
        }
    }

    pub fn to_response(&self) -> Result<String, Error> {
        let count = self.count;
        match &*self.response {
            Response::Id(state) => {
                let resp = CountId {
                    count,
                    response: state.to_owned(),
                };
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::All(state) => {
                let resp = CountAll {
                    count,
                    response: state.to_owned(),
                };
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::Order(state) => {
                let resp = CountOrder {
                    count,
                    response: state.to_owned(),
                };
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::GroupBy(state) => {
                let resp = CountGroupBy {
                    count,
                    response: state.to_owned(),
                };
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::OrderedGroupBy(state) => {
                let resp = CountOrderedGroupBy {
                    count,
                    response: state.to_owned(),
                };
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::OptionOrder(state) => {
                let resp = CountOptionOrder {
                    count,
                    response: state.to_owned(),
                };
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::OptionGroupBy(state) => {
                let resp = CountOptionGroupBy {
                    count,
                    response: state.to_owned(),
                };
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::OptionSelect(state) => {
                let resp = CountOptionSelect {
                    count,
                    response: state.to_owned(),
                };
                debug! ("Called Response::OptionSelect");
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::CheckValues(state) => {
                let resp = CountCheckValues {
                    count,
                    response: state.to_owned(),
                };
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::TimeRange(state) => {
                let resp = CountTimeRange {
                    count,
                    response: state.to_owned(),
                };
                #[cfg(feature = "json")]
                return Ok(serde_json::to_string(&resp)?);

                #[cfg(not(feature = "json"))]
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            Response::DateSelect(state) => {
                let resp = CountDateSelect {
                    count,
                    response: state.to_owned(),
                };
                Ok(ron::ser::to_string_pretty(&resp, pretty_config_output())?)
            }
            _ => Err(Error::Unknown),
        }
    }
}

#[cfg(feature = "json_pure")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Response {
    Id(HashMap<String, Types>),
    Intersect(HashMap<String, Types>),
    Difference(HashMap<String, Types>),
    Union(HashMap<String, Types>),
    All(BTreeMap<Uuid, HashMap<String, TypesSelfDescribing>>),
    Order(Vec<(Uuid, HashMap<String, Types>)>),
    GroupBy(HashMap<String, BTreeMap<Uuid, HashMap<String, Types>>>),
    OrderedGroupBy(HashMap<String, Vec<(Uuid, HashMap<String, Types>)>>),
    OptionOrder(Vec<(Uuid, Option<HashMap<String, Types>>)>),
    OptionGroupBy(HashMap<String, BTreeMap<Uuid, Option<HashMap<String, Types>>>>),
    OptionSelect(BTreeMap<Uuid, Option<HashMap<String, TypesSelfDescribing>>>),
    CheckValues(HashMap<String, bool>),
    TimeRange(BTreeMap<DateTime<Utc>, HashMap<String, Types>>),
    WithCount(CountResponse),
    DateSelect(HashMap<String, HashMap<String, Types>>),
    Join(Vec<HashMap<String, TypesSelfDescribing>>),
}

#[cfg(not(feature = "json_pure"))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Response {
    Id(HashMap<String, Types>),
    Intersect(HashMap<String, Types>),
    Difference(HashMap<String, Types>),
    Union(HashMap<String, Types>),
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
    Join(Vec<HashMap<String, Types>>),
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
#[cfg(feature = "json_pure")]
impl From<BTreeMap<Uuid, Option<HashMap<String, Types>>>> for Response {
    fn from(map: BTreeMap<Uuid, Option<HashMap<String, Types>>>) -> Self {
        debug! ("Called from");
        Self::OptionSelect (map.into_iter ()
            .map (|(bk, bv)| {
                if let Some (h) = bv {
                    (bk, Some (h.into_iter().map(|(k, v)| (k, TypesSelfDescribing::from (v)) ).collect::<HashMap<String,TypesSelfDescribing>> ()))
                }
                else
                {
                    (bk, None)
                }
            }).collect ())
    }
}

#[cfg(feature = "json_pure")]


#[cfg(not(feature = "json_pure"))]
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

#[cfg(feature = "json_pure")]
impl From<BTreeMap<Uuid, HashMap<String, Types>>> for Response {
    fn from(map: BTreeMap<Uuid, HashMap<String, Types>>) -> Self {
        Self::All(map.into_iter ()
            .map (|(bk, bv)| {
                (bk, bv.into_iter().map(|(k, v)| (k, TypesSelfDescribing::from (v)) ).collect::<HashMap<String,TypesSelfDescribing>> ())
            }).collect ())
    }
}

#[cfg(not(feature = "json_pure"))]
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

#[cfg(feature = "json_pure")]
impl Response {
    pub fn parse(
        self,
        key: String,
        ent_b: &(String, String),
        vec: &mut Vec<HashMap<String, TypesSelfDescribing>>,
        b_hash: HashMap<TypesSelfDescribing, Vec<HashMap<String, TypesSelfDescribing>>>,
    ) -> bool {
        /*
        let b_hash_sd: HashMap<TypesSelfDescribing, Vec<HashMap<String, TypesSelfDescribing>>> = b_hash.into_iter ().map (|(bk, bv)| {
            (
                TypesSelfDescribing::from (bk),
                bv.into_iter ().map (|e| {
                    e.into_iter ().map (|(k,v)| { ( k, TypesSelfDescribing::from (v) ) }).collect ()
                }).collect ()
            )
        }).collect ();
        */
        match self {
            Response::OptionGroupBy(_)
            | Response::CheckValues(_)
            | Response::TimeRange(_)
            | Response::WithCount(_)
            | Response::Id(_)
            | Response::Intersect(_)
            | Response::Difference(_)
            | Response::Union(_)
            | Response::GroupBy(_)
            | Response::OrderedGroupBy(_)
            | Response::Join(_)
            | Response::DateSelect(_) => {
                return false;
            }
            Response::All(state) => {
                state.into_iter().for_each(|(_, s)| {
                    let s_sd = s.into_iter ().map (|(k,v)| { (k, TypesSelfDescribing::from (v) ) }).collect::<HashMap<String, TypesSelfDescribing>> ();
                    let type_key = s_sd.get(&key).unwrap_or(&TypesSelfDescribing::Nil);
                    let entities = b_hash.get(type_key);
                    if let Some(v) = entities {
                        for ent in v {
                            let mut s = s_sd.clone();
                            for entry in ent
                                .iter()
                                .filter(|(k, _)| *k != "tx_time" && *k != &ent_b.1)
                            {
                                let entry_name = if s.contains_key(entry.0) {
                                    format!("{}:{}", entry.0, ent_b.0)
                                } else {
                                    entry.0.to_owned()
                                };
                                s.insert(entry_name, entry.1.to_owned());
                            }
                            vec.push(s.to_owned());
                        }
                    }
                });
            }
            Response::Order(state) => {
                state.into_iter().for_each(|(_, s)| {
                    let s_sd = s.into_iter ().map (|(k,v)| { (k, TypesSelfDescribing::from (v) ) }).collect::<HashMap<String, TypesSelfDescribing>> ();
                    let type_key = s_sd.get(&key).unwrap_or(&TypesSelfDescribing::Nil);
                    let entities = b_hash.get(type_key);
                    if let Some(v) = entities {
                        for ent in v {
                            let mut s = s_sd.clone();
                            for entry in ent
                                .iter()
                                .filter(|(k, _)| *k != "tx_time" && *k != &ent_b.1)
                            {
                                let entry_name = if s.contains_key(entry.0) {
                                    format!("{}:{}", entry.0, ent_b.0)
                                } else {
                                    entry.0.to_owned()
                                };
                                s.insert(entry_name, entry.1.to_owned());
                            }
                            vec.push(s.to_owned());
                        }
                    }
                });
            }
            Response::OptionOrder(state) => {
                state.into_iter().for_each(|(_, s)| {
                    if let Some(s) = s {
                        let s_sd = s.into_iter ().map (|(k,v)| { (k, TypesSelfDescribing::from (v) ) }).collect::<HashMap<String, TypesSelfDescribing>> ();
                        let type_key = s_sd.get(&key).unwrap_or(&TypesSelfDescribing::Nil);
                        let entities = b_hash.get(type_key);
                        if let Some(v) = entities {
                            for ent in v {
                                let mut s = s_sd.clone();
                                for entry in ent
                                    .iter()
                                    .filter(|(k, _)| *k != "tx_time" && *k != &ent_b.1)
                                {
                                    let entry_name = if s.contains_key(entry.0) {
                                        format!("{}:{}", entry.0, ent_b.0)
                                    } else {
                                        entry.0.to_owned()
                                    };
                                    s.insert(entry_name, entry.1.to_owned());
                                }
                                vec.push(s.to_owned());
                            }
                        }
                    }
                });
            }
            Response::OptionSelect(state) => {
                state.into_iter().for_each(|(_, s)| {
                    if let Some(s) = s {
                        let type_key = s.get(&key).unwrap_or(&TypesSelfDescribing::Nil);
                        let entities = b_hash.get(type_key);
                        if let Some(v) = entities {
                            for ent in v {
                                let mut s = s.clone();
                                for entry in ent
                                    .iter()
                                    .filter(|(k, _)| *k != "tx_time" && *k != &ent_b.1)
                                {
                                    let entry_name = if s.contains_key(entry.0) {
                                        entry.0.to_owned()
                                    } else {
                                        format!("{}:{}", entry.0, ent_b.0)
                                    };
                                    s.insert(entry_name, entry.1.to_owned());
                                }
                                vec.push(s.to_owned());
                            }
                        }
                    }
                });
            }
        }
        true
    }

    pub fn hash(self, key: &str) -> Option<HashMap<TypesSelfDescribing, Vec<HashMap<String, TypesSelfDescribing>>>> {
        let mut hm = HashMap::new();
        match self {
            Response::All(state) => {
                state.into_iter().for_each(|(_, s)| {
                    let s_sd = s.into_iter ().map (|(k,v)| { (k, TypesSelfDescribing::from (v) ) }).collect::<HashMap<String, TypesSelfDescribing>> ();
                    let entry = hm
                        .entry(s_sd.get(key).unwrap_or(&TypesSelfDescribing::Nil).to_owned())
                        .or_insert(Vec::new());
                    (*entry).push(s_sd);
                });
            }
            Response::Order(state) => {
                state.into_iter().for_each(|(_, s)| {
                    let s_sd = s.into_iter ().map (|(k,v)| { (k, TypesSelfDescribing::from (v) ) }).collect::<HashMap<String, TypesSelfDescribing>> ();
                    let entry = hm
                        .entry(s_sd.get(key).unwrap_or(&TypesSelfDescribing::Nil).to_owned())
                        .or_insert(Vec::new());
                    (*entry).push(s_sd);
                });
            }
            Response::OptionOrder(state) => {
                state.into_iter().for_each(|(_, so)| {
                    if let Some(s) = so {
                        let s_sd = s.into_iter ().map (|(k,v)| { (k, TypesSelfDescribing::from (v) ) }).collect::<HashMap<String, TypesSelfDescribing>> ();
                        let entry = hm
                            .entry(s_sd.get(key).unwrap_or(&TypesSelfDescribing::Nil).to_owned())
                            .or_insert(Vec::new());
                        (*entry).push(s_sd);
                    }
                });
            }
            Response::OptionSelect(state) => {
                state.into_iter().for_each(|(_, s)| {
                    if let Some(s) = s {
                        let entry = hm
                            .entry(s.get(key).unwrap_or(&TypesSelfDescribing::Nil).to_owned())
                            .or_insert(Vec::new());
                        (*entry).push(s);
                    }
                });
            }
            Response::OptionGroupBy(_)
            | Response::CheckValues(_)
            | Response::Join(_)
            | Response::TimeRange(_)
            | Response::WithCount(_)
            | Response::Id(_)
            | Response::Intersect(_)
            | Response::Difference(_)
            | Response::Union(_)
            | Response::GroupBy(_)
            | Response::OrderedGroupBy(_)
            | Response::DateSelect(_) => {
                return None;
            }
        }

        Some(hm)
    }

    pub fn to_string(&self) -> Result<String, Error> {
        match self {
            Response::Id(state) => Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?),
            Response::Intersect(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::Difference(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::Union(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::CheckValues(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::TimeRange(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::All(state) => { debug! ("Called Response.to_string for Response::All"); Ok(serde_json::to_string (&state)?) },
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
            Response::WithCount(state) => state.to_response(),
            Response::OptionSelect(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::DateSelect(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::Join(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
        }
    }
}

#[cfg(not(feature = "json_pure"))]
impl Response {
    pub fn parse(
        self,
        key: String,
        ent_b: &(String, String),
        vec: &mut Vec<HashMap<String, Types>>,
        b_hash: HashMap<Types, Vec<HashMap<String, Types>>>,
    ) -> bool {
        match self {
            Response::OptionGroupBy(_)
            | Response::CheckValues(_)
            | Response::TimeRange(_)
            | Response::WithCount(_)
            | Response::Id(_)
            | Response::Intersect(_)
            | Response::Difference(_)
            | Response::Union(_)
            | Response::GroupBy(_)
            | Response::OrderedGroupBy(_)
            | Response::Join(_)
            | Response::DateSelect(_) => {
                return false;
            }
            Response::All(state) => {
                state.into_iter().for_each(|(_, s)| {
                    let type_key = s.get(&key).unwrap_or(&Types::Nil);
                    let entities = b_hash.get(type_key);
                    if let Some(v) = entities {
                        for ent in v {
                            let mut s = s.clone();
                            for entry in ent
                                .iter()
                                .filter(|(k, _)| *k != "tx_time" && *k != &ent_b.1)
                            {
                                let entry_name = if s.contains_key(entry.0) {
                                    format!("{}:{}", entry.0, ent_b.0)
                                } else {
                                    entry.0.to_owned()
                                };
                                s.insert(entry_name, entry.1.to_owned());
                            }
                            vec.push(s.to_owned());
                        }
                    }
                });
            }
            Response::Order(state) => {
                state.into_iter().for_each(|(_, s)| {
                    let type_key = s.get(&key).unwrap_or(&Types::Nil);
                    let entities = b_hash.get(type_key);
                    if let Some(v) = entities {
                        for ent in v {
                            let mut s = s.clone();
                            for entry in ent
                                .iter()
                                .filter(|(k, _)| *k != "tx_time" && *k != &ent_b.1)
                            {
                                let entry_name = if s.contains_key(entry.0) {
                                    format!("{}:{}", entry.0, ent_b.0)
                                } else {
                                    entry.0.to_owned()
                                };
                                s.insert(entry_name, entry.1.to_owned());
                            }
                            vec.push(s.to_owned());
                        }
                    }
                });
            }
            Response::OptionOrder(state) => {
                state.into_iter().for_each(|(_, s)| {
                    if let Some(s) = s {
                        let type_key = s.get(&key).unwrap_or(&Types::Nil);
                        let entities = b_hash.get(type_key);
                        if let Some(v) = entities {
                            for ent in v {
                                let mut s = s.clone();
                                for entry in ent
                                    .iter()
                                    .filter(|(k, _)| *k != "tx_time" && *k != &ent_b.1)
                                {
                                    let entry_name = if s.contains_key(entry.0) {
                                        format!("{}:{}", entry.0, ent_b.0)
                                    } else {
                                        entry.0.to_owned()
                                    };
                                    s.insert(entry_name, entry.1.to_owned());
                                }
                                vec.push(s.to_owned());
                            }
                        }
                    }
                });
            }
            Response::OptionSelect(state) => {
                state.into_iter().for_each(|(_, s)| {
                    if let Some(s) = s {
                        let type_key = s.get(&key).unwrap_or(&Types::Nil);
                        let entities = b_hash.get(type_key);
                        if let Some(v) = entities {
                            for ent in v {
                                let mut s = s.clone();
                                for entry in ent
                                    .iter()
                                    .filter(|(k, _)| *k != "tx_time" && *k != &ent_b.1)
                                {
                                    let entry_name = if s.contains_key(entry.0) {
                                        entry.0.to_owned()
                                    } else {
                                        format!("{}:{}", entry.0, ent_b.0)
                                    };
                                    s.insert(entry_name, entry.1.to_owned());
                                }
                                vec.push(s.to_owned());
                            }
                        }
                    }
                });
            }
        }
        true
    }

    pub fn hash(self, key: &str) -> Option<HashMap<Types, Vec<HashMap<String, Types>>>> {
        let mut hm = HashMap::new();
        match self {
            Response::All(state) => {
                state.into_iter().for_each(|(_, s)| {
                    let entry = hm
                        .entry(s.get(key).unwrap_or(&Types::Nil).to_owned())
                        .or_insert(Vec::new());
                    (*entry).push(s);
                });
            }
            Response::Order(state) => {
                state.into_iter().for_each(|(_, s)| {
                    let entry = hm
                        .entry(s.get(key).unwrap_or(&Types::Nil).to_owned())
                        .or_insert(Vec::new());
                    (*entry).push(s);
                });
            }
            Response::OptionOrder(state) => {
                state.into_iter().for_each(|(_, s)| {
                    if let Some(s) = s {
                        let entry = hm
                            .entry(s.get(key).unwrap_or(&Types::Nil).to_owned())
                            .or_insert(Vec::new());
                        (*entry).push(s);
                    }
                });
            }
            Response::OptionSelect(state) => {
                state.into_iter().for_each(|(_, s)| {
                    if let Some(s) = s {
                        let entry = hm
                            .entry(s.get(key).unwrap_or(&Types::Nil).to_owned())
                            .or_insert(Vec::new());
                        (*entry).push(s);
                    }
                });
            }
            Response::OptionGroupBy(_)
            | Response::CheckValues(_)
            | Response::Join(_)
            | Response::TimeRange(_)
            | Response::WithCount(_)
            | Response::Id(_)
            | Response::Intersect(_)
            | Response::Difference(_)
            | Response::Union(_)
            | Response::GroupBy(_)
            | Response::OrderedGroupBy(_)
            | Response::DateSelect(_) => {
                return None;
            }
        }

        Some(hm)
    }

    pub fn to_string(&self) -> Result<String, Error> {
        match self {
            Response::Id(state) => Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?),
            Response::Intersect(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::Difference(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::Union(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
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
            Response::WithCount(state) => state.to_response(),
            Response::OptionSelect(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::DateSelect(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
            Response::Join(state) => {
                Ok(ron::ser::to_string_pretty(&state, pretty_config_output())?)
            }
        }
    }
}
