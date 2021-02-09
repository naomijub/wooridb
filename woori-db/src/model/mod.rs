pub(crate) mod error;
pub(crate) mod wql;

use actix::prelude::*;
use actix_web::web;
use serde::{Deserialize, Serialize};
use std::{io::Error, sync::{Arc, Mutex, atomic::AtomicUsize}};

use crate::{actors::wql::Executor, repository::local::{EncryptContext, LocalContext, UniquenessContext}};

pub type DataLocalContext = web::Data<Arc<Mutex<LocalContext>>>;
pub type DataUniquenessContext= web::Data<Arc<Mutex<UniquenessContext>>>;
pub type DataEncryptContext = web::Data<Arc<Mutex<EncryptContext>>>;
pub type DataAtomicUsize = web::Data<AtomicUsize>;
pub type  DataU32 = web::Data<u32>;
pub type DataExecutor = web::Data<Addr<Executor>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
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
