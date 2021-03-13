use std::collections::{BTreeMap, HashMap};

use actix::prelude::*;
use uuid::Uuid;
use wql::Types;

use crate::{
    core::pretty_config_inner,
    io::write::{local_data, offset_counter},
    model::{error::Error, DataRegister},
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
    pub data: BTreeMap<String, BTreeMap<Uuid, (DataRegister, HashMap<String, Types>)>>,
}

impl LocalData {
    pub fn new(
        data: BTreeMap<String, BTreeMap<Uuid, (DataRegister, HashMap<String, Types>)>>,
    ) -> Self {
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
