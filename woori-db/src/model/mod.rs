pub(crate) mod error;
pub(crate) mod wql;

use actix::prelude::*;
use std::io::Error;

use crate::actors::wql::Executor;
#[derive(Debug, Clone)]
pub struct DataRegister {
    pub file_name: String,
    pub offset: usize,
    pub bytes_length: usize,
}

impl Message for DataRegister {
    type Result = Result<String, Error>;
}

impl Handler<DataRegister> for Executor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: DataRegister, _: &mut Self::Context) -> Self::Result {
        use crate::io::read::read_log;
        read_log(msg)
    }
}
