use std::collections::{BTreeMap, HashMap};

use uuid::Uuid;

pub type LocalContext = BTreeMap<String, BTreeMap<Uuid, Types>>;

#[derive(Debug)]
pub enum Types {
    Char(char),
    Integer(isize),
    String(String),
    Uuid(Uuid),
    Float(f64),
    Boolean(bool),
    Vector(Vec<String>),
    Map(HashMap<String, String>),
    //DateTime
    Nil,
}
