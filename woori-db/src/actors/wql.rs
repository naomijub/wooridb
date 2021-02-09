use actix::prelude::*;
use chrono::{DateTime, Utc};
use std::io::Error;
use uuid::Uuid;

use crate::core::wql::{
    create_entity, delete_entity_content, evict_entity_content, evict_entity_id_content,
    insert_entity_content, update_content_entity_content, update_set_entity_content,
};

pub struct Executor;

impl Actor for Executor {
    type Context = Context<Self>;
}

impl Executor {
    pub fn new() -> Self {
        Executor {}
    }
}

pub struct CreateEntity {
    name: String,
}

impl CreateEntity {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}

impl Message for CreateEntity {
    type Result = Result<usize, Error>;
}

impl Handler<CreateEntity> for Executor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, msg: CreateEntity, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let entity = create_entity(&msg.name);
        write_to_log(&entity)
    }
}

pub struct InsertEntityContent {
    pub name: String,
    pub content: String,
}

impl InsertEntityContent {
    pub fn new(name: &str, content: &str) -> Self {
        Self {
            name: name.to_owned(),
            content: content.to_owned(),
        }
    }
}

impl Message for InsertEntityContent {
    type Result = Result<(DateTime<Utc>, Uuid, usize), Error>;
}

impl Handler<InsertEntityContent> for Executor {
    type Result = Result<(DateTime<Utc>, Uuid, usize), Error>;

    fn handle(&mut self, msg: InsertEntityContent, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let (date, uuid, content) = insert_entity_content(&msg);
        Ok((date, uuid, write_to_log(&content)?))
    }
}

pub struct UpdateSetEntityContent {
    pub name: String,
    pub current_state: String,
    pub content_log: String,
    pub id: Uuid,
    pub previous_registry: String,
}

impl UpdateSetEntityContent {
    pub fn new(
        name: &str,
        current_state: &str,
        content_log: &str,
        id: Uuid,
        previous_registry: &str,
    ) -> Self {
        Self {
            name: name.to_owned(),
            content_log: content_log.to_owned(),
            current_state: current_state.to_owned(),
            id,
            previous_registry: previous_registry.to_owned(),
        }
    }
}

impl Message for UpdateSetEntityContent {
    type Result = Result<(DateTime<Utc>, usize), Error>;
}

impl Handler<UpdateSetEntityContent> for Executor {
    type Result = Result<(DateTime<Utc>, usize), Error>;

    fn handle(&mut self, msg: UpdateSetEntityContent, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let (date, content) = update_set_entity_content(&msg);
        Ok((date, write_to_log(&content)?))
    }
}

// I know it is duplicated
pub struct UpdateContentEntityContent {
    pub name: String,
    pub current_state: String,
    pub content_log: String,
    pub id: Uuid,
    pub previous_registry: String,
}

impl UpdateContentEntityContent {
    pub fn new(
        name: &str,
        current_state: &str,
        content_log: &str,
        id: Uuid,
        previous_registry: &str,
    ) -> Self {
        Self {
            name: name.to_owned(),
            content_log: content_log.to_owned(),
            current_state: current_state.to_owned(),
            id,
            previous_registry: previous_registry.to_owned(),
        }
    }
}

impl Message for UpdateContentEntityContent {
    type Result = Result<(DateTime<Utc>, usize), Error>;
}

impl Handler<UpdateContentEntityContent> for Executor {
    type Result = Result<(DateTime<Utc>, usize), Error>;

    fn handle(&mut self, msg: UpdateContentEntityContent, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let (date, content) = update_content_entity_content(&msg);
        Ok((date, write_to_log(&content)?))
    }
}

pub struct DeleteId {
    pub name: String,
    pub content_log: String,
    pub uuid: Uuid,
    pub previous_registry: String,
}

impl DeleteId {
    pub fn new(name: &str, content_log: &str, uuid: Uuid, previous_registry: &str) -> Self {
        Self {
            name: name.to_owned(),
            content_log: content_log.to_owned(),
            uuid,
            previous_registry: previous_registry.to_owned(),
        }
    }
}

impl Message for DeleteId {
    type Result = Result<(DateTime<Utc>, usize), Error>;
}

impl Handler<DeleteId> for Executor {
    type Result = Result<(DateTime<Utc>, usize), Error>;

    fn handle(&mut self, msg: DeleteId, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let (date, content) = delete_entity_content(&msg);
        Ok((date, write_to_log(&content)?))
    }
}

pub struct EvictEntity {
    pub name: String,
}

impl EvictEntity {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
}

impl Message for EvictEntity {
    type Result = Result<usize, Error>;
}

impl Handler<EvictEntity> for Executor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, msg: EvictEntity, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let content = evict_entity_content(&msg.name);
        Ok(write_to_log(&content)?)
    }
}

pub struct EvictEntityId {
    pub name: String,
    pub id: Uuid,
}

impl EvictEntityId {
    pub fn new(name: &str, id: Uuid) -> Self {
        Self {
            name: name.to_owned(),
            id,
        }
    }
}

impl Message for EvictEntityId {
    type Result = Result<usize, Error>;
}

impl Handler<EvictEntityId> for Executor {
    type Result = Result<usize, Error>;

    fn handle(&mut self, msg: EvictEntityId, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let content = evict_entity_id_content(&msg);
        Ok(write_to_log(&content)?)
    }
}
