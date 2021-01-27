pub enum Action {
    Create,
    Insert,
    Read,
    Update,
    Error
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
       match *self {
           Action::Read => write!(f, "READ"),
           Action::Create => write!(f, "CREATE"),
           Action::Insert => write!(f, "INSERT"),
           Action::Update => write!(f, "UPDATE"),
           Action::Error => write!(f, "Error"),
       }
    }
}

impl From<String> for Action {
    fn from(val: String) -> Self {
        match val.as_str() {
            "READ" => Action::Read,
            "CREATE" => Action::Create,
            "INSERT" => Action::Insert,
            "UPDATE" => Action::Update,
            _ => Action::Error
        }
    }
}