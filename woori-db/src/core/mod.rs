extern crate wql as ewql;
use std::collections::HashMap;

use chrono::{DateTime, Utc};
use ewql::Types;
use ron::ser::PrettyConfig;

use crate::model::error::Error;

pub(crate) mod query;
pub(crate) mod registry;
pub(crate) mod wql;

pub fn pretty_config_output() -> PrettyConfig {
    PrettyConfig::new()
        .with_separate_tuple_members(true)
        .with_decimal_floats(true)
        .with_indentor(" ".to_string())
        .with_new_line("\n".to_string())
}

pub fn pretty_config_inner() -> PrettyConfig {
    PrettyConfig::new()
        .with_indentor("".to_string())
        .with_new_line("".to_string())
}

pub fn tx_time(content: &HashMap<String, Types>) -> Result<DateTime<Utc>, Error> {
    if content.contains_key("tx_time") {
        return Err(Error::KeyTxTimeNotAllowed);
    }
    Ok(Utc::now())
}
