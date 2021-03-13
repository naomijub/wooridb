use crate::{model::error::Error, schemas::pretty_config};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountResponse {
    response: String,
    count: usize,
}

impl CountResponse {
    pub fn to_response(count: usize, response: String) -> Result<String, Error> {
        let resp = Self { count, response };
        Ok(ron::ser::to_string_pretty(&resp, pretty_config())?)
    }
}
