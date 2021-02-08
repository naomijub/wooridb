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

#[cfg(test)]
pub fn assert_not_content(pat: &str) {
    use chrono::prelude::*;
    let utc: DateTime<Utc> = Utc::now();
    let date_log = utc.format("%Y_%m_%d.log").to_string();

    let mut file = OpenOptions::new().read(true).open(date_log).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(!s.contains(pat));
}

#[cfg(test)]
pub fn assert_uniques(pat: &str) {
    let mut file = OpenOptions::new().read(true).open("uniques.log").unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}

#[cfg(test)]
pub fn assert_encrypt(pat: &str) {
    let mut file = OpenOptions::new().read(true).open("encrypt.log").unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}

pub fn read_log(registry: DataRegister) -> Result<String, Error> {
    let mut file = OpenOptions::new().read(true).open(registry.file_name)?;
    file.seek(SeekFrom::Start(registry.offset as u64))?;
    let mut res = String::with_capacity(registry.bytes_length);
    file.take(registry.bytes_length as u64)
        .read_to_string(&mut res)?;

    Ok(res)
}

#[cfg(test)]
mod test {
    use std::{fs::OpenOptions, io::Write};

    use super::*;
    use crate::model::DataRegister;

    #[test]
    fn read_log_range() {
        let log_size = write_new();
        let data = DataRegister {
            file_name: "read_test.log".to_string(),
            offset: 30,
            bytes_length: log_size - 58,
        };

        let log = read_log(data).unwrap();
        assert_eq!(log, "i am too lazy to create.");
    }

    fn write_new() -> usize {
        let log =
            "this is a very long text that i am too lazy to create. Guess it is enough already.";
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("read_test.log")
            .unwrap();

        file.write(log.as_bytes()).unwrap()
    }
}
