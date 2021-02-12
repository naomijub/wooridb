use chrono::{DateTime, Utc};
use std::io::{BufRead, BufReader, Error, Seek, SeekFrom};
use std::{fs::OpenOptions, io::Write};

pub fn write_to_log(log: &str) -> Result<(usize, bool), Error> {
    let utc: DateTime<Utc> = Utc::now();
    let date_log = utc.format("%Y_%m_%d.log").to_string();

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .read(true)
        .open(date_log)?;
    let buff = BufReader::new(file.try_clone()?);
    let written_bytes = file.write(log.as_bytes())?;

    Ok((written_bytes, buff.lines().count() == 0))
}
pub fn write_to_uniques(log: &str) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("uniques.log")?;

    let _ = file.write(log.as_bytes())?;

    Ok(())
}

pub fn local_data(log: &str) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("local_data.log")?;

    let _ = file.seek(SeekFrom::Start(0));
    file.write_all(log.as_bytes())?;

    Ok(())
}

pub fn unique_data(log: &str) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("unique_data.log")?;

    let _ = file.seek(SeekFrom::Start(0));
    file.write_all(log.as_bytes())?;

    Ok(())
}

pub fn offset_counter(log: usize) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("offset_counter.log")?;

    let _ = file.seek(SeekFrom::Start(0));
    file.write_all(log.to_string().as_bytes())?;

    Ok(())
}

pub fn write_to_encrypts(log: &str) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("encrypt.log")?;

    let _ = file.write(log.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::io::read::{
        assert_content, assert_local_data, assert_offset, assert_unique_data, assert_uniques,
    };
    #[test]
    fn write_unique() {
        let _ = write_to_uniques("oh crazy unique log");
        assert_uniques("oh crazy unique log");
    }

    #[test]
    fn write_log() {
        let _ = write_to_log("oh crazy log");
        assert_content("oh crazy log");
    }

    #[test]
    fn offset_counter_test() {
        let _ = offset_counter(5_usize);
        assert_offset("5");
    }

    #[test]
    fn local_data_test() {
        let _ = local_data("some crazy date here");
        assert_local_data("some crazy date here");
    }

    #[test]
    fn unique_data_test() {
        let _ = unique_data("some crazy date here");
        assert_unique_data("some crazy date here");
    }
}
