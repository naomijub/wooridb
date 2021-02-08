use std::collections::{BTreeMap, HashMap, HashSet};

use uuid::Uuid;

use crate::model::DataRegister;

pub type LocalContext = BTreeMap<String, BTreeMap<Uuid, DataRegister>>;
pub type UniquenessContext = BTreeMap<String, HashMap<String, HashSet<String>>>;
pub type EncryptContext = BTreeMap<String, HashSet<String>>;
