use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::OpenOptions,
    io::{Error, Read, Seek, SeekFrom},
};

use rayon::prelude::*;
use uuid::Uuid;
use wql::Types;

use crate::model::error;
use crate::{actors::encrypts::WriteWithEncryption, model::DataRegister};

#[cfg(test)]
pub fn assert_content(pat: &str) {
    use chrono::prelude::*;
    let utc: DateTime<Utc> = Utc::now();
    let date_log = utc.format("data/%Y_%m_%d.log").to_string();

    let mut file = OpenOptions::new().read(true).open(date_log).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}

#[cfg(test)]
pub fn assert_not_content(pat: &str) {
    use chrono::prelude::*;
    let utc: DateTime<Utc> = Utc::now();
    let date_log = utc.format("data/%Y_%m_%d.log").to_string();

    let mut file = OpenOptions::new().read(true).open(date_log).unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(!s.contains(pat));
}

#[cfg(test)]
pub fn assert_uniques(pat: &str) {
    let mut file = OpenOptions::new()
        .read(true)
        .open("data/uniques.log")
        .unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}

#[cfg(test)]
pub fn assert_offset(pat: &str) {
    let mut file = OpenOptions::new()
        .read(true)
        .open("data/offset_counter.log")
        .unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}

#[cfg(test)]
pub fn assert_local_data(pat: &str) {
    let mut file = OpenOptions::new()
        .read(true)
        .open("data/local_data.log")
        .unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}

#[cfg(test)]
pub fn assert_unique_data(pat: &str) {
    let mut file = OpenOptions::new()
        .read(true)
        .open("data/unique_data.log")
        .unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}

#[cfg(test)]
pub fn assert_encrypt(pat: &str) {
    let mut file = OpenOptions::new()
        .read(true)
        .open("data/encrypt.log")
        .unwrap();
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

pub fn read_date_log(date_log: String) -> Result<String, Error> {
    let mut file = OpenOptions::new().read(true).open(date_log)?;
    file.seek(SeekFrom::Start(0))?;
    let mut res = String::new();
    file.read_to_string(&mut res)?;

    Ok(res)
}

pub fn offset() -> Result<usize, error::Error> {
    #[cfg(not(feature = "test_read"))]
    let path = "data/offset_counter.log";
    #[cfg(feature = "test_read")]
    let path = "data/offset_counter.txt";
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    Ok(s.parse::<usize>()
        .map_err(|_| error::Error::FailedToParseState)?)
}

pub fn local_data(
) -> Result<BTreeMap<String, BTreeMap<Uuid, (DataRegister, HashMap<String, Types>)>>, error::Error>
{
    #[cfg(not(feature = "test_read"))]
    let path = "data/local_data.log";
    #[cfg(feature = "test_read")]
    let path = "data/local_data.txt";
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    let data: Result<
        BTreeMap<String, BTreeMap<Uuid, (DataRegister, HashMap<String, Types>)>>,
        error::Error,
    > = match ron::de::from_str(&s) {
        Ok(x) => Ok(x),
        Err(_) => Err(error::Error::FailedToParseState),
    };

    data
}

pub fn unique_data() -> Result<BTreeMap<String, HashMap<String, HashSet<String>>>, error::Error> {
    #[cfg(not(feature = "test_read"))]
    let path = "data/unique_data.log";
    #[cfg(feature = "test_read")]
    let path = "data/unique_data.txt";
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut s = String::new();
    file.read_to_string(&mut s)?;

    let data: Result<BTreeMap<String, HashMap<String, HashSet<String>>>, error::Error> =
        match ron::de::from_str(&s) {
            Ok(x) => Ok(x),
            Err(_) => Err(error::Error::FailedToParseState),
        };

    data
}

pub fn encryption() -> Result<BTreeMap<String, HashSet<String>>, error::Error> {
    #[cfg(not(feature = "test_read"))]
    let path = "data/encrypt.log";
    #[cfg(feature = "test_read")]
    let path = "data/encrypt.txt";
    let mut file = OpenOptions::new().read(true).open(path)?;
    let mut s = String::from('[');
    file.read_to_string(&mut s)?;
    s.push(']');
    let s = s.replace(")(", "),(");

    let data: Result<Vec<WriteWithEncryption>, error::Error> = match ron::de::from_str(&s) {
        Ok(x) => Ok(x),
        Err(_) => Err(error::Error::FailedToParseState),
    };

    let data = data?
        .into_par_iter()
        .map(|enc| {
            (
                enc.entity,
                enc.encrypts.into_iter().collect::<HashSet<String>>(),
            )
        })
        .collect::<BTreeMap<String, HashSet<String>>>();

    Ok(data)
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
            file_name: "data/read_test.log".to_string(),
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
            .open("data/read_test.log")
            .unwrap();

        file.write(log.as_bytes()).unwrap()
    }

    #[cfg(feature = "test_read")]
    #[test]
    fn encryption_test() {
        let encrypt = encryption().unwrap();
        let s = format!("{:?}", encrypt);

        assert!(s.contains("encrypt_ent"));
        assert!(s.contains("encrypt_ent2"));
        assert!(s.contains("name"));
        assert!(s.contains("cpf"));
    }

    #[cfg(feature = "test_read")]
    #[test]
    fn offset_test() {
        let offset = offset();

        assert_eq!(offset.unwrap(), 701);
    }

    #[cfg(feature = "test_read")]
    #[test]
    fn local_data_test() {
        let local_data = local_data();

        assert!(local_data.is_ok());
        assert_eq!(
                format!("{:?}", local_data), 
                "Ok({\"encrypt_ent\": {}, \"encrypt_ent2\": {}, \"hello\": {50e68bc1-0c3b-4ffc-93be-46e57f59b415: (DataRegister { file_name: \"2021_02_10.log\", offset: 447, bytes_length: 153 }, {})}, \"oh_yeah\": {27367bd0-1966-4005-a8b5-5e323e1c3524: (DataRegister { file_name: \"2021_02_10.log\", offset: 180, bytes_length: 247 }, {})}})"
            );
    }

    #[cfg(feature = "test_read")]
    #[test]
    fn unique_data_test() {
        let unique_data = unique_data();

        assert!(unique_data.is_ok());
        assert_eq!(
                format!("{:?}", unique_data), 
                "Ok({\"uniq2_ent2\": {\"id\": {\"Integer(4234)\", \"Integer(734)\"}, \"rg\": {\"Precise(\\\"42356546\\\")\", \"Precise(\\\"123456789\\\")\"}}, \"uniq_ent\": {\"cpf\": {\"Precise(\\\"42356546\\\")\", \"Precise(\\\"423560546\\\")\"}, \"snn\": {}}})"
            );
    }
}
