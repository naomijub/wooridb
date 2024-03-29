use actix::prelude::*;

use crate::{
    core::pretty_config_inner,
    io::write::{local_data, offset_counter},
    model::error::Error,
    repository::local::LocalContext,
};

use super::wql::Executor;

pub struct OffsetCounter {
    pub offset: usize,
}

impl OffsetCounter {
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }
}

impl Message for OffsetCounter {
    type Result = Result<(), Error>;
}

impl Handler<OffsetCounter> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: OffsetCounter, _: &mut Self::Context) -> Self::Result {
        Ok(offset_counter(msg.offset)?)
    }
}

pub struct LocalData {
    pub data: LocalContext,
}

impl LocalData {
    pub fn new(data: LocalContext) -> Self {
        Self { data }
    }
}

impl Message for LocalData {
    type Result = Result<(), Error>;
}

impl Handler<LocalData> for Executor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: LocalData, _: &mut Self::Context) -> Self::Result {
        let data_str = ron::ser::to_string_pretty(&msg.data, pretty_config_inner())?;
        Ok(local_data(&data_str)?)
    }
}
