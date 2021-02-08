use language_parser::read_symbol;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;

mod language_parser;
mod logic;
mod select;
#[cfg(test)]
mod test;

use logic::{read_map, read_match_args};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Wql {
    CreateEntity(String, Vec<String>, Vec<String>),
    Insert(String, Entity),
    UpdateContent(String, Entity, Uuid),
    UpdateSet(String, Entity, Uuid),
    Delete(String, String),
    MatchUpdate(String, Entity, Uuid, MatchCondition),
    Evict(String, Option<Uuid>),
    Select(String, ToSelect, Option<Uuid>),
    SelectIds(String, ToSelect, Vec<Uuid>),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ToSelect {
    All,
    Keys(Vec<String>),
}

pub type Entity = HashMap<String, Types>;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Types {
    Char(char),
    Integer(isize),
    String(String),
    Uuid(Uuid),
    Float(f64),
    Boolean(bool),
    Vector(Vec<Types>),
    Map(HashMap<String, Types>),
    Hash(String),
    //DateTime
    Nil,
}

impl Types {
    pub fn default_values(&self) -> Types {
        match self {
            Types::Char(_) => Types::Char(' '),
            Types::Integer(_) => Types::Integer(0),
            Types::String(_) => Types::String(String::new()),
            Types::Uuid(_) => Types::Uuid(Uuid::new_v4()),
            Types::Float(_) => Types::Float(0f64),
            Types::Boolean(_) => Types::Boolean(false),
            Types::Vector(_) => Types::Vector(Vec::new()),
            Types::Map(_) => Types::Map(HashMap::new()),
            Types::Hash(_) => Types::Hash(String::new()),
            Types::Nil => Types::Nil,
        }
    }

    pub fn to_hash(&self, cost: Option<u32>) -> Result<Types, String> {
        use bcrypt::{hash, DEFAULT_COST};
        let value = match self {
            Types::Char(c) => format!("{}", c),
            Types::Integer(i) => format!("{}", i),
            Types::String(s) => format!("{}", s),
            Types::Uuid(id) => format!("{}", id),
            Types::Float(f) => format!("{}", f),
            Types::Boolean(b) => format!("{}", b),
            Types::Vector(vec) => format!("{:?}", vec),
            Types::Map(map) => format!("{:?}", map),
            Types::Hash(_) => return Err(String::from("Hash cannot be hashed")),
            Types::Nil => return Err(String::from("Nil cannot be hashed")),
        };
        match hash(
            &value,
            if cost.is_some() {
                cost.unwrap()
            } else {
                DEFAULT_COST
            },
        ) {
            Ok(s) => Ok(Types::Hash(s)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum MatchCondition {
    All(Vec<MatchCondition>),
    Any(Vec<MatchCondition>),
    Eq(String, Types),
    NotEq(String, Types),
    GEq(String, Types),
    G(String, Types),
    LEq(String, Types),
    L(String, Types),
}

pub(crate) fn tokenize(wql: &str) -> std::str::Chars {
    wql.chars()
}

impl std::str::FromStr for Wql {
    type Err = String;

    /// Parses a `&str` that contains an Edn into `Result<Edn, EdnError>`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = tokenize(s.trim_start());
        let wql = parse(tokens.next(), &mut tokens)?;
        Ok(wql)
    }
}

pub(crate) fn parse(c: Option<char>, chars: &mut std::str::Chars) -> Result<Wql, String> {
    c.map_or_else(
        || Err(String::from("Empty WQL")),
        |ch| read_symbol(ch, chars),
    )
}
