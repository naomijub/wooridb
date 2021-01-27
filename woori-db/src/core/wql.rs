use crate::model::wql::Action;

pub fn create_entity(entity: &String) -> String {
    format!("{}|{},", Action::Create, entity)
}