use chrono::{DateTime, Utc};
use language_parser::read_symbol;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, hash::Hash};
use std::{collections::HashMap, str::FromStr};
use uuid::Uuid;
mod join;
mod language_parser;
mod logic;
mod relation;
mod select;
#[cfg(test)]
mod test;
mod where_clause;

pub use logic::parse_value as parse_types;
use logic::{integer_decode, read_map, read_match_args};
pub use relation::{Relation, RelationType};
pub use where_clause::{Clause, Function, Value};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Wql {
    CreateEntity(String, Vec<String>, Vec<String>),
    Insert(String, Entity, Option<Uuid>),
    UpdateContent(String, Entity, Uuid),
    UpdateSet(String, Entity, Uuid),
    Delete(String, String),
    MatchUpdate(String, Entity, Uuid, MatchCondition),
    Evict(String, Option<Uuid>),
    Select(String, ToSelect, Option<Uuid>, HashMap<String, Algebra>),
    SelectWhen(String, ToSelect, Option<Uuid>, String),
    SelectWhenRange(String, Uuid, String, String),
    SelectIds(String, ToSelect, Vec<Uuid>, HashMap<String, Algebra>),
    SelectWhere(String, ToSelect, Vec<Clause>, HashMap<String, Algebra>),
    CheckValue(String, Uuid, HashMap<String, String>),
    RelationQuery(Vec<Wql>, Relation, RelationType),
    Join((String, String), (String, String), Vec<Wql>),
}

pub use select::{Algebra, Order};

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ToSelect {
    All,
    Keys(Vec<String>),
}

pub type Entity = HashMap<String, Types>;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
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

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn tokenize(wql: &str) -> std::str::Chars {
    wql.chars()
}

impl std::str::FromStr for Wql {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = tokenize(s.trim_start());
        let wql = parse(tokens.next(), &mut tokens)?;
        Ok(wql)
    }
}

#[allow(clippy::redundant_pub_crate)]
pub(crate) fn parse(c: Option<char>, chars: &mut std::str::Chars) -> Result<Wql, String> {
    c.map_or_else(
        || Err(String::from("Empty WQL")),
        |ch| read_symbol(ch, chars),
    )
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    Precise(String),
    DateTime(DateTime<Utc>),
    Nil,
}

impl Types {
    pub fn default_values(&self) -> Self {
        match self {
            Self::Char(_) => Self::Char(' '),
            Self::Integer(_) => Self::Integer(0),
            Self::String(_) => Self::String(String::new()),
            Self::Uuid(_) => Self::Uuid(Uuid::new_v4()),
            Self::Float(_) => Self::Float(0_f64),
            Self::Boolean(_) => Self::Boolean(false),
            Self::Vector(_) => Self::Vector(Vec::new()),
            Self::Map(_) => Self::Map(HashMap::new()),
            Self::Hash(_) => Self::Hash(String::new()),
            Self::Precise(_) => Self::Precise(String::from("0")),
            Self::DateTime(_) => Self::DateTime(Utc::now()),
            Self::Nil => Self::Nil,
        }
    }

    pub fn to_hash(&self, cost: Option<u32>) -> Result<Self, String> {
        use bcrypt::{hash, DEFAULT_COST};
        let value = match self {
            Self::Char(c) => format!("{}", c),
            Self::Integer(i) => format!("{}", i),
            Self::String(s) => s.to_string(),
            Self::DateTime(date) => date.to_string(),
            Self::Uuid(id) => format!("{}", id),
            Self::Float(f) => format!("{:?}", integer_decode(f.to_owned())),
            Self::Boolean(b) => format!("{}", b),
            Self::Vector(vec) => format!("{:?}", vec),
            Self::Map(map) => format!("{:?}", map),
            Self::Precise(p) => p.to_string(),
            Self::Hash(_) => return Err(String::from("Hash cannot be hashed")),
            Self::Nil => return Err(String::from("Nil cannot be hashed")),
        };
        match hash(value, cost.map_or(DEFAULT_COST, |c| c)) {
            Ok(s) => Ok(Self::Hash(s)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    pub const fn is_hash(&self) -> bool {
        matches!(self, Self::Hash(_))
    }
}

impl Eq for Types {}
impl PartialOrd for Types {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => Some(a.cmp(b)),

            (Self::Float(a), Self::Float(b)) => Some(if a > b {
                Ordering::Greater
            } else {
                Ordering::Less
            }),
            (Self::Integer(a), Self::Float(b)) => Some(if &(*a as f64) > b {
                Ordering::Greater
            } else {
                Ordering::Less
            }),
            (Self::Float(a), Self::Integer(b)) => Some(if a > &(*b as f64) {
                Ordering::Greater
            } else {
                Ordering::Less
            }),
            (Self::Char(a), Self::Char(b)) => Some(a.cmp(b)),
            (Self::String(a), Self::String(b)) | (Self::Precise(a), Self::Precise(b)) => {
                Some(a.cmp(b))
            }
            (Self::Uuid(a), Self::Uuid(b)) => Some(a.cmp(b)),
            (Self::Boolean(a), Self::Boolean(b)) => Some(a.cmp(b)),
            (Self::Vector(a), Self::Vector(b)) => Some(a.len().cmp(&b.len())),
            _ => None,
        }
    }
}

// UNSAFE
impl Hash for Types {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Char(t) => t.hash(state),
            Self::Integer(t) => t.hash(state),
            Self::String(t) => t.hash(state),
            Self::Uuid(t) => t.hash(state),
            Self::Float(t) => {
                let int_t = integer_decode(t.to_owned());
                int_t.hash(state)
            }
            Self::Boolean(t) => t.hash(state),
            Self::Vector(t) => t.hash(state),
            Self::Map(t) => t.iter().fold((), |acc, (k, v)| {
                k.hash(state);
                v.hash(state);
                acc
            }),
            Self::Hash(t) => t.hash(state),
            Self::Precise(t) => t.hash(state),
            Self::DateTime(t) => t.hash(state),
            Self::Nil => "".hash(state),
        }
    }
}
