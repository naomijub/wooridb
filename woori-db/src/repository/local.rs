use std::collections::BTreeMap;

use uuid::Uuid;

use crate::model::DataRegister;

pub type LocalContext = BTreeMap<String, BTreeMap<Uuid, DataRegister>>;
