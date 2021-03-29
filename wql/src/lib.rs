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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ToSelect {
    All,
    Keys(Vec<String>),
}

pub type Entity = HashMap<String, Types>;

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
    pub fn default_values(&self) -> Types {
        match self {
            Types::Char(_) => Types::Char(' '),
            Types::Integer(_) => Types::Integer(0),
            Types::String(_) => Types::String(String::new()),
            Types::Uuid(_) => Types::Uuid(Uuid::new_v4()),
            Types::Float(_) => Types::Float(0_f64),
            Types::Boolean(_) => Types::Boolean(false),
            Types::Vector(_) => Types::Vector(Vec::new()),
            Types::Map(_) => Types::Map(HashMap::new()),
            Types::Hash(_) => Types::Hash(String::new()),
            Types::Precise(_) => Types::Precise(String::from("0")),
            Types::DateTime(_) => Types::DateTime(Utc::now()),
            Types::Nil => Types::Nil,
        }
    }

    pub fn to_hash(&self, cost: Option<u32>) -> Result<Types, String> {
        use bcrypt::{hash, DEFAULT_COST};
        let value = match self {
            Types::Char(c) => format!("{}", c),
            Types::Integer(i) => format!("{}", i),
            Types::String(s) => s.to_string(),
            Types::DateTime(date) => date.to_string(),
            Types::Uuid(id) => format!("{}", id),
            Types::Float(f) => format!("{:?}", integer_decode(f.to_owned())),
            Types::Boolean(b) => format!("{}", b),
            Types::Vector(vec) => format!("{:?}", vec),
            Types::Map(map) => format!("{:?}", map),
            Types::Precise(p) => p.to_string(),
            Types::Hash(_) => return Err(String::from("Hash cannot be hashed")),
            Types::Nil => return Err(String::from("Nil cannot be hashed")),
        };
        match hash(&value, cost.map_or(DEFAULT_COST, |c| c)) {
            Ok(s) => Ok(Types::Hash(s)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    pub fn is_hash(&self) -> bool {
        matches!(self, Types::Hash(_))
    }
}

impl Eq for Types {}
// UNSAFE
impl PartialOrd for Types {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Types::Integer(a), Types::Integer(b)) => Some(a.cmp(b)),
            (Types::Float(a), Types::Float(b)) => {
                let (mant_a, exp_a, sig_a) = integer_decode(a.to_owned());
                let (mant_b, exp_b, sig_b) = integer_decode(b.to_owned());
                Some(
                    sig_a
                        .cmp(&sig_b)
                        .then(mant_a.cmp(&mant_b))
                        .then(exp_a.cmp(&exp_b)),
                )
            }
            (Types::Integer(a), Types::Float(b)) => Some(if &(*a as f64) > b {
                Ordering::Greater
            } else {
                Ordering::Less
            }),
            (Types::Float(a), Types::Integer(b)) => Some(if a > &(*b as f64) {
                Ordering::Greater
            } else {
                Ordering::Less
            }),
            (Types::Char(a), Types::Char(b)) => Some(a.cmp(b)),
            (Types::String(a), Types::String(b)) | (Types::Precise(a), Types::Precise(b)) => {
                Some(a.cmp(b))
            }
            (Types::Uuid(a), Types::Uuid(b)) => Some(a.cmp(b)),
            (Types::Boolean(a), Types::Boolean(b)) => Some(a.cmp(b)),
            (Types::Vector(a), Types::Vector(b)) => Some(a.len().cmp(&b.len())),
            _ => None,
        }
    }
}

// UNSAFE
impl Hash for Types {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Types::Char(t) => t.hash(state),
            Types::Integer(t) => t.hash(state),
            Types::String(t) => t.hash(state),
            Types::Uuid(t) => t.hash(state),
            Types::Float(t) => {
                let int_t = integer_decode(t.to_owned());
                int_t.hash(state)
            }
            Types::Boolean(t) => t.hash(state),
            Types::Vector(t) => t.hash(state),
            Types::Map(t) => t.into_iter().fold((), |acc, (k, v)| {
                k.hash(state);
                v.hash(state);
                acc
            }),
            Types::Hash(t) => t.hash(state),
            Types::Precise(t) => t.hash(state),
            Types::DateTime(t) => t.hash(state),
            Types::Nil => "".hash(state),
        }
    }
}
