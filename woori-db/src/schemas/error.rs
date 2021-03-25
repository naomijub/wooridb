use crate::core::pretty_config_output;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    error_type: String,
    error_message: String,
}

impl Response {
    pub fn new(error_type: String, error_message: String) -> Self {
        Self {
            error_type,
            error_message,
        }
    }

    pub fn write(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = ron::ser::to_string_pretty(self, pretty_config_output())
            .unwrap_or_else(|_| "SERVER ERROR".to_owned());
        write!(f, "{}", output)
    }
}
