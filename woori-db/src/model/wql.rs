use std::collections::HashMap;

use wql::{MatchCondition, Types, ID};

#[derive(Debug, PartialEq)]
pub enum Action {
    CreateEntity,
    Insert,
    Read,
    UpdateSet,
    UpdateContent,
    Delete,
    EvictEntity,
    EvictEntityId,
    Error,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Action::Read => write!(f, "READ"),
            Action::CreateEntity => write!(f, "CREATE_ENTITY"),
            Action::Insert => write!(f, "INSERT"),
            Action::UpdateSet => write!(f, "UPDATE_SET"),
            Action::UpdateContent => write!(f, "UPDATE_CONTENT"),
            Action::Delete => write!(f, "DELETE"),
            Action::EvictEntity => write!(f, "EVICT_ENTITY"),
            Action::EvictEntityId => write!(f, "EVICT_ENTITY_ID"),
            Action::Error => write!(f, "Error"),
        }
    }
}

impl From<String> for Action {
    fn from(val: String) -> Self {
        match val.as_str() {
            "READ" => Action::Read,
            "CREATE_ENTITY" => Action::CreateEntity,
            "INSERT" => Action::Insert,
            "DELETE" => Action::Delete,
            "UPDATE_SET" => Action::UpdateSet,
            "UPDATE_CONTENT" => Action::UpdateContent,
            "EVICT_ENTITY" => Action::EvictEntity,
            "EVICT_ENTITY_ID" => Action::EvictEntityId,
            _ => Action::Error,
        }
    }
}

pub struct MatchUpdateArgs {
    pub entity: String,
    pub content: HashMap<String, Types>,
    pub id: ID,
    pub conditions: MatchCondition,
}

impl MatchUpdateArgs {
    pub fn new(
        entity: String,
        content: HashMap<String, Types>,
        id: ID,
        conditions: MatchCondition,
    ) -> Self {
        Self {
            entity,
            content,
            id,
            conditions,
        }
    }
}

pub struct UpdateArgs {
    pub entity: String,
    pub content: HashMap<String, Types>,
    pub id: ID,
}

impl UpdateArgs {
    pub fn new(entity: String, content: HashMap<String, Types>, id: ID) -> Self {
        Self {
            entity,
            content,
            id,
        }
    }
}

pub struct InsertArgs {
    pub entity: String,
    pub content: HashMap<String, Types>,
    pub uuid: Option<ID>,
}

impl InsertArgs {
    pub fn new(entity: String, content: HashMap<String, Types>, uuid: Option<ID>) -> Self {
        Self {
            entity,
            content,
            uuid,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_str() {
        assert_eq!(Action::from(String::from("READ")), Action::Read);
        assert_eq!(
            Action::from(String::from("CREATE_ENTITY")),
            Action::CreateEntity
        );
        assert_eq!(Action::from(String::from("INSERT")), Action::Insert);
        assert_eq!(Action::from(String::from("DELETE")), Action::Delete);
        assert_eq!(Action::from(String::from("UPDATE_SET")), Action::UpdateSet);
        assert_eq!(
            Action::from(String::from("UPDATE_CONTENT")),
            Action::UpdateContent
        );
    }
}
