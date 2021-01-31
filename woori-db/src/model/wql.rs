pub enum Action {
    CreateEntity,
    Insert,
    Read,
    UpdateSet,
    UpdateContent,
    Delete,
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
            _ => Action::Error,
        }
    }
}
