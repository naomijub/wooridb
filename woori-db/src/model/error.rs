use std::io;

use uuid::Uuid;
use wql::Types;

use crate::schemas::error::ErrorResponse;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    QueryFormat(String),
    EntityAlreadyCreated(String),
    EntityNotCreated(String),
    SerializationError(ron::Error),
    UuidNotCreatedForEntity(String, Uuid),
    FailedToParseState,
    FailedToParseRegistry,
    UnkwonCondition,
    FailedMatchCondition,
    DuplicatedUnique(String, String, Types),
    SelectBadRequest,
    NonSelectQuery,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::QueryFormat(s) => {
                ErrorResponse::new(String::from("QueryFormat"), format!("{:?}", s)).write(f)
            }
            Error::Io(e) => ErrorResponse::new(String::from("IO"), format!("{:?}", e)).write(f),
            Error::EntityAlreadyCreated(e) => ErrorResponse::new(
                String::from("EntityAlreadyCreated"),
                format!("Entity `{}` already created", e),
            )
            .write(f),
            Error::EntityNotCreated(e) => ErrorResponse::new(
                String::from("EntityNotCreated"),
                format!("Entity `{}` not created", e),
            )
            .write(f),
            Error::SerializationError(e) => {
                ErrorResponse::new(String::from("SerializationError"), format!("{:?}", e)).write(f)
            }
            Error::UuidNotCreatedForEntity(s, id) => ErrorResponse::new(
                String::from("UuidNotCreatedForEntity"),
                format!("Uuid {:?} not created for entity {}", id, s),
            )
            .write(f),
            Error::FailedToParseState => ErrorResponse::new(
                String::from("FailedToParseState"),
                format!("Failed to parse state"),
            )
            .write(f),
            Error::FailedToParseRegistry => ErrorResponse::new(
                String::from("FailedToParseRegistry"),
                format!("Failed to parse registry"),
            )
            .write(f),
            Error::DuplicatedUnique(entity, key, t) => ErrorResponse::new(
                String::from("DuplicatedUnique"),
                format!(
                    "key `{}` in entity `{}` already contains value `{:?}`",
                    key, entity, t
                ),
            )
            .write(f),
            Error::UnkwonCondition => ErrorResponse::new(
                String::from("UnkwonCondition"),
                format!("UNKNOWN MATCH CONDITION"),
            )
            .write(f),
            Error::FailedMatchCondition => ErrorResponse::new(
                String::from("FailedMatchCondition"),
                format!("One or more MATCH CONDITIONS failed"),
            )
            .write(f),
            Error::SelectBadRequest => ErrorResponse::new(
                String::from("SelectBadRequest"),
                format!("SELECT expressions are handled by `/wql/query` endpoint"),
            )
            .write(f),
            Error::NonSelectQuery => ErrorResponse::new(
                String::from("NonSelectQuery"),
                format!("Non-SELECT expressions are handled by `/wql/tx` endpoint"),
            )
            .write(f),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}
