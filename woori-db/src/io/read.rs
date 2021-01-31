use std::{
    fs::OpenOptions,
    io::{Error, Read, Seek, SeekFrom},
};

use crate::model::DataRegister;

#[cfg(test)]
pub fn assert_content(pat: &str) {
    use chrono::prelude::*;
    let utc: DateTime<Utc> = Utc::now();
    let date_log = utc.format("%Y_%m_%d.log").to_string();

    let mut file = OpenOptions::new().read(true).open(date_log).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}

pub fn read_log(registry: DataRegister) -> Result<String, Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(registry.file_name)
        .unwrap();
    file.seek(SeekFrom::Start(registry.offset as u64))?;
    let mut res = String::with_capacity(registry.bytes_length);
    file.take(registry.bytes_length as u64)
        .read_to_string(&mut res)?;

    Ok(res)
}
