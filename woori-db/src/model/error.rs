use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    QueryFormat(String),
    EntityAlreadyCreated(String),
    EntityNotCreated(String),
    SerializationError(ron::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::QueryFormat(s) => write!(f, "{:?}", s),
            Error::Io(e) => write!(f, "{:?}", e),
            Error::EntityAlreadyCreated(e) => write!(f, "Entity `{}` already created", e),
            Error::EntityNotCreated(e) => write!(f, "Entity `{}` not created", e),
            Error::SerializationError(e) => write!(f, "{:?}", e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}
