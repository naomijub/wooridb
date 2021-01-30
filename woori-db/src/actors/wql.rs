use actix::prelude::*;
use std::io::Error;

use crate::core::wql::create_entity;

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
    pub name: String,
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
