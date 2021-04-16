use std::collections::BTreeMap;

use wql::ID;

use crate::{
    model::{error::Error, DataLocalContext},
    repository::local::StateInfo,
};

pub fn get_registries(
    entity: &str,
    local_data: &DataLocalContext,
) -> Result<BTreeMap<ID, StateInfo>, Error> {
    let local_data = if let Ok(guard) = local_data.lock() {
        guard
    } else {
        return Err(Error::LockData);
    };
    let registries = if let Some(id_to_registries) = local_data.get(entity) {
        id_to_registries
    } else {
        return Err(Error::EntityNotCreated(entity.to_owned()));
    }
    .to_owned();
    Ok(registries)
}
