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
    type Result = Result<(usize, bool), Error>;
}

impl Handler<CreateEntity> for Executor {
    type Result = Result<(usize, bool), Error>;

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
    type Result = Result<(DateTime<Utc>, Uuid, usize, bool), Error>;
}

impl Handler<InsertEntityContent> for Executor {
    type Result = Result<(DateTime<Utc>, Uuid, usize, bool), Error>;

    fn handle(&mut self, msg: InsertEntityContent, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let (date, uuid, content) = insert_entity_content(&msg);
        let (bytes_written, is_empty) = write_to_log(&content)?;
        Ok((date, uuid, bytes_written, is_empty))
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
    type Result = Result<(DateTime<Utc>, usize, bool), Error>;
}

impl Handler<UpdateSetEntityContent> for Executor {
    type Result = Result<(DateTime<Utc>, usize, bool), Error>;

    fn handle(&mut self, msg: UpdateSetEntityContent, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let (date, content) = update_set_entity_content(&msg);
        let (bytes_written, is_empty) = write_to_log(&content)?;
        Ok((date, bytes_written, is_empty))
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
    type Result = Result<(DateTime<Utc>, usize, bool), Error>;
}

impl Handler<UpdateContentEntityContent> for Executor {
    type Result = Result<(DateTime<Utc>, usize, bool), Error>;

    fn handle(&mut self, msg: UpdateContentEntityContent, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let (date, content) = update_content_entity_content(&msg);
        let (bytes_written, is_empty) = write_to_log(&content)?;
        Ok((date, bytes_written, is_empty))
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
    type Result = Result<(DateTime<Utc>, usize, bool), Error>;
}

impl Handler<DeleteId> for Executor {
    type Result = Result<(DateTime<Utc>, usize, bool), Error>;

    fn handle(&mut self, msg: DeleteId, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let (date, content) = delete_entity_content(&msg);
        let (bytes_written, is_empty) = write_to_log(&content)?;
        Ok((date, bytes_written, is_empty))
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
    type Result = Result<(usize, bool), Error>;
}

impl Handler<EvictEntity> for Executor {
    type Result = Result<(usize, bool), Error>;

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
    type Result = Result<(usize, bool), Error>;
}

impl Handler<EvictEntityId> for Executor {
    type Result = Result<(usize, bool), Error>;

    fn handle(&mut self, msg: EvictEntityId, _: &mut Self::Context) -> Self::Result {
        use crate::io::write::write_to_log;
        let content = evict_entity_id_content(&msg);
        Ok(write_to_log(&content)?)
    }
}

#[cfg(test)]
mod test {
    use actix::Actor;

    use crate::io::read;

    use super::{
        CreateEntity, DeleteId, EvictEntity, EvictEntityId, Executor, InsertEntityContent,
        UpdateSetEntityContent,
    };

    #[actix_rt::test]
    async fn create_test() {
        let create = CreateEntity {
            name: String::from("create-my-entity"),
        };
        let actor = Executor::new().start();

        let resp = actor.send(create).await.unwrap();
        assert!(resp.is_ok());
        read::assert_content("CREATE_ENTITY|create-my-entity;")
    }

    #[actix_rt::test]
    async fn insert_test() {
        let insert = InsertEntityContent {
            name: String::from("insert-my-entity"),
            content: String::from("this is the content"),
        };
        let actor = Executor::new().start();

        let resp = actor.send(insert).await.unwrap();
        assert!(resp.is_ok());
        read::assert_content("INSERT|");
        read::assert_content("insert-my-entity");
        read::assert_content("this is the content");
    }

    #[actix_rt::test]
    async fn update_set_test() {
        let uuid = uuid::Uuid::new_v4();
        let update_set = UpdateSetEntityContent::new(
            "update-set-my-entity",
            "this is the content",
            "this is the current state",
            uuid,
            "this is the previous registry",
        );
        let actor = Executor::new().start();

        let resp = actor.send(update_set).await.unwrap();
        assert!(resp.is_ok());
        read::assert_content("UPDATE_SET|");
        read::assert_content("update-set-my-entity");
        read::assert_content("this is the content");
        read::assert_content(&uuid.to_string());
        read::assert_content("this is the current state");
        read::assert_content("this is the previous registry");
    }

    #[actix_rt::test]
    async fn update_content_test() {
        let uuid = uuid::Uuid::new_v4();
        let update_content = UpdateSetEntityContent::new(
            "update-content-my-entity",
            "this is the content",
            "this is the current state",
            uuid,
            "this is the previous registry",
        );
        let actor = Executor::new().start();

        let resp = actor.send(update_content).await.unwrap();
        assert!(resp.is_ok());
        read::assert_content("UPDATE_SET|");
        read::assert_content("update-content-my-entity");
        read::assert_content(&uuid.to_string());
    }

    #[actix_rt::test]
    async fn delete_test() {
        let uuid = uuid::Uuid::new_v4();
        let update_content = DeleteId::new(
            "delete-my-entity",
            "this is the content",
            uuid,
            "this is the previous registry",
        );
        let actor = Executor::new().start();

        let resp = actor.send(update_content).await.unwrap();
        assert!(resp.is_ok());
        read::assert_content("DELETE|");
        read::assert_content("delete-my-entity");
        read::assert_content(&uuid.to_string());
    }

    #[actix_rt::test]
    async fn evict_test() {
        let evict = EvictEntity::new("evict-my-entity");
        let actor = Executor::new().start();

        let resp = actor.send(evict).await.unwrap();
        assert!(resp.is_ok());
        read::assert_content("EVICT_ENTITY|");
        read::assert_content("evict-my-entity");
    }

    #[actix_rt::test]
    async fn evict_id_test() {
        let uuid = uuid::Uuid::new_v4();
        let evict = EvictEntityId::new("evict-id-my-entity", uuid);
        let actor = Executor::new().start();

        let resp = actor.send(evict).await.unwrap();
        assert!(resp.is_ok());
        read::assert_content("EVICT_ENTITY_ID|");
        read::assert_content("evict-id-my-entity");
        read::assert_content(&uuid.to_string());
    }
}
