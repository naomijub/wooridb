use crate::schemas::pretty_config;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    error_type: String,
    error_message: String,
}

impl ErrorResponse {
    pub fn new(error_type: String, error_message: String) -> Self {
        Self {
            error_type,
            error_message,
        }
    }

    pub fn write(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output =
            ron::ser::to_string_pretty(self, pretty_config()).unwrap_or("SERVER ERROR".to_string());
        write!(f, "{}", output)
    }
}
