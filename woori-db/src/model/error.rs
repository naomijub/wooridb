use actix::MailboxError;
use actix_web::{error, HttpResponse};
use std::io;

use uuid::Uuid;
use wql::Types;

use crate::schemas::error::Response;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    QueryFormat(String),
    InvalidQuery,
    EntityAlreadyCreated(String),
    EntityNotCreated(String),
    EntityNotCreatedWithUniqueness(String),
    Serialization(ron::Error),
    #[cfg(feature = "json")]
    SerdeJson(serde_json::Error),
    UuidNotCreatedForEntity(String, Uuid),
    FailedToParseState,
    FailedToParseRegistry,
    UnknownCondition,
    FailedMatchCondition,
    DuplicatedUnique(String, String, Types),
    SelectBadRequest,
    NonSelectQuery,
    ActixMailbox(MailboxError),
    LockData,
    KeyTxTimeNotAllowed,
    Ron(ron::Error),
    InvalidUuid(uuid::Error),
    UpdateContentEncryptKeys(Vec<String>),
    CheckNonEncryptedKeys(Vec<String>),
    DateTimeParse(chrono::ParseError),
    FailedToParseDate,
    AdminNotConfigured,
    #[allow(dead_code)]
    AuthorizationBadRequest,
    AuthenticationBadRequest,
    AuthenticationBadRequestBody(String),
    FailedToCreateUser,
    FailedToDeleteUsers,
    Unknown,
}

pub fn error_to_http(e: &Error) -> HttpResponse {
    match e {
        Error::Io(_)
        | Error::Serialization(_)
        | Error::FailedToParseState
        | Error::FailedToParseRegistry
        | Error::UnknownCondition
        | Error::ActixMailbox(_)
        | Error::Ron(_)
        | Error::DateTimeParse(_)
        | Error::FailedToParseDate => HttpResponse::InternalServerError().body(e.to_string()),
        Error::QueryFormat(_)
        | Error::InvalidQuery
        | Error::DuplicatedUnique(_, _, _)
        | Error::EntityNotCreated(_)
        | Error::EntityNotCreatedWithUniqueness(_)
        | Error::UuidNotCreatedForEntity(_, _)
        | Error::InvalidUuid(_)
        | Error::UpdateContentEncryptKeys(_)
        | Error::CheckNonEncryptedKeys(_)
        | Error::FailedToCreateUser
        | Error::FailedToDeleteUsers
        | Error::KeyTxTimeNotAllowed => HttpResponse::BadRequest().body(e.to_string()),
        Error::EntityAlreadyCreated(_) => HttpResponse::UnprocessableEntity().body(e.to_string()),
        #[cfg(feature = "json")]
        Error::SerdeJson(_) => HttpResponse::InternalServerError().body(e.to_string()),
        Error::FailedMatchCondition => HttpResponse::PreconditionFailed().body(e.to_string()),
        Error::NonSelectQuery => HttpResponse::MethodNotAllowed().body(e.to_string()),
        Error::LockData => HttpResponse::ServiceUnavailable().body(e.to_string()),
        Error::AdminNotConfigured | Error::AuthorizationBadRequest | Error::Unknown => {
            HttpResponse::Unauthorized().body(e.to_string())
        }
        Error::AuthenticationBadRequest | Error::AuthenticationBadRequestBody(_) => {
            HttpResponse::Forbidden().body(e.to_string())
        }
        Error::SelectBadRequest => HttpResponse::MethodNotAllowed().body(e.to_string()),
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::QueryFormat(s) => {
                Response::new(String::from("QueryFormat"), format!("{:?}", s)).write(f)
            }
            Error::InvalidQuery => {
                Response::new(
                    String::from("InvalidQuery"), 
                    "Only single value queries are allowed, so key `ID` is required and keys `WHEN AT` are optional".to_string()
                ).write(f)
            }
            Error::Io(e) => Response::new(String::from("IO"), format!("{:?}", e)).write(f),
            Error::EntityAlreadyCreated(e) => Response::new(
                String::from("EntityAlreadyCreated"),
                format!("Entity `{}` already created", e),
            )
            .write(f),
            Error::EntityNotCreated(e) => Response::new(
                String::from("EntityNotCreated"),
                format!("Entity `{}` not created", e),
            )
            .write(f),
            Error::EntityNotCreatedWithUniqueness(e) => Response::new(
                String::from("EntityNotCreatedWithUniqueness"),
                format!("Entity `{}` not created", e),
            )
            .write(f),
            Error::Serialization(e) => {
                Response::new(String::from("Serialization"), format!("{:?}", e)).write(f)
            }
            #[cfg(feature = "json")]
            Error::SerdeJson(e) => {
                Response::new(String::from("SerdeJson"), format!("{:?}", e)).write(f)
            }
            Error::UuidNotCreatedForEntity(s, id) => Response::new(
                String::from("UuidNotCreatedForEntity"),
                format!("Uuid {:?} not created for entity {}", id, s),
            )
            .write(f),
            Error::FailedToParseState => Response::new(
                String::from("FailedToParseState"),
                "Failed to parse state".to_string(),
            )
            .write(f),
            Error::FailedToParseRegistry => Response::new(
                String::from("FailedToParseRegistry"),
                "Failed to parse registry".to_string(),
            )
            .write(f),
            Error::DuplicatedUnique(entity, key, t) => Response::new(
                String::from("DuplicatedUnique"),
                format!(
                    "key `{}` in entity `{}` already contains value `{:?}`",
                    key, entity, t
                ),
            )
            .write(f),
            Error::UnknownCondition => Response::new(
                String::from("UnknownCondition"),
                "UNKNOWN MATCH CONDITION".to_string(),
            )
            .write(f),
            Error::FailedMatchCondition => Response::new(
                String::from("FailedMatchCondition"),
                "One or more MATCH CONDITIONS failed".to_string(),
            )
            .write(f),
            Error::SelectBadRequest => Response::new(
                String::from("SelectBadRequest"),
                "SELECT expressions are handled by `/wql/query` endpoint".to_string(),
            )
            .write(f),
            Error::NonSelectQuery => Response::new(
                String::from("NonSelectQuery"),
                "Non-SELECT expressions are handled by `/wql/tx` endpoint".to_string(),
            )
            .write(f),
            Error::ActixMailbox(r) => {
                Response::new(String::from("ActixMailbox"), format!("{:?}", r)).write(f)
            }
            Error::LockData => Response::new(
                String::from("LockData"),
                "System was not able to get a lock on data".to_string(),
            )
            .write(f),
            Error::Ron(e) => Response::new(String::from("Ron"), format!("{:?}", e)).write(f),
            Error::InvalidUuid(e) => {
                Response::new(String::from("InvalidUuid"), format!("{:?}", e)).write(f)
            }
            Error::UpdateContentEncryptKeys(keys) => Response::new(
                String::from("UpdateContentEncryptKeys"),
                format!(
                    "Encrypted keys cannont be updated with UPDATE CONTENT: {:?}",
                    keys
                ),
            )
            .write(f),
            Error::CheckNonEncryptedKeys(keys) => Response::new(
                String::from("CheckNonEncryptedKeys"),
                format!("CHECK can only verify encrypted keys: {:?}", keys),
            )
            .write(f),
            Error::DateTimeParse(e) => Response::new(
                String::from("DateTimeParse"),
                format!("Date parse error: {:?}", e),
            )
            .write(f),
            Error::FailedToParseDate => Response::new(
                String::from("FailedToParseDate"),
                "Log date parse error".to_string(),
            )
            .write(f),
            Error::AdminNotConfigured => Response::new(
                String::from("AdminNotConfigured"),
                "Admin credentials not configured".to_string(),
            )
            .write(f),
            Error::AuthorizationBadRequest => Response::new(
                String::from("AuthorizationBadRequest"),
                "Bad request at authorizing endpoint".to_string(),
            )
            .write(f),
            Error::AuthenticationBadRequest => Response::new(
                String::from("AuthenticationBadRequest"),
                "Bad request at authenticating endpoint".to_string(),
            )
            .write(f),
            Error::AuthenticationBadRequestBody(error) => Response::new(
                String::from("AuthenticationBadRequest"),
                format!("Bad request: {}", error),
            )
            .write(f),
            Error::FailedToCreateUser => Response::new(
                String::from("FailedToCreateUser"),
                "Failed to create requested user".to_string(),
            )
            .write(f),
            Error::FailedToDeleteUsers => Response::new(
                String::from("FailedToDeleteUsers"),
                "Failed to delete requested users".to_string(),
            )
            .write(f),
            Error::Unknown => Response::new(
                String::from("Unknown"),
                "Request credentials failed".to_string(),
            )
            .write(f),
            Error::KeyTxTimeNotAllowed => Response::new(
                String::from("KeyTxTimeNotAllowed"),
                "Key `tx_time` is not allowed".to_string(),
            )
            .write(f),
        }
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeJson(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<MailboxError> for Error {
    fn from(error: MailboxError) -> Self {
        Error::ActixMailbox(error)
    }
}

impl From<ron::Error> for Error {
    fn from(error: ron::Error) -> Self {
        Error::Ron(error)
    }
}

impl From<uuid::Error> for Error {
    fn from(error: uuid::Error) -> Self {
        Error::InvalidUuid(error)
    }
}

impl error::ResponseError for Error {}
