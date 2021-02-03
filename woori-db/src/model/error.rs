use std::io;

use uuid::Uuid;
use wql::Types;

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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::QueryFormat(s) => write!(f, "{:?}", s),
            Error::Io(e) => write!(f, "{:?}", e),
            Error::EntityAlreadyCreated(e) => write!(f, "Entity `{}` already created", e),
            Error::EntityNotCreated(e) => write!(f, "Entity `{}` not created", e),
            Error::SerializationError(e) => write!(f, "{:?}", e),
            Error::UuidNotCreatedForEntity(s, id) => {
                write!(f, "Uuid {:?} not created for entity {}", id, s)
            }
            Error::FailedToParseState => write!(f, "Failed to parse state"),
            Error::FailedToParseRegistry => write!(f, "Failed to parse registry"),
            Error::DuplicatedUnique(entity, key, t) => write!(
                f,
                "key `{}` in entity `{}` already contains value `{:?}`",
                key, entity, t
            ),
            Error::UnkwonCondition => write!(f, "UNKNOWN MATCH CONDITION"),
            Error::FailedMatchCondition => write!(f, "One or more MATCH CONDITIONS failed"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}
