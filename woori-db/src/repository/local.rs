use std::collections::{BTreeMap};

use uuid::Uuid;

pub type LocalContext = BTreeMap<String, BTreeMap<Uuid, String>>;
