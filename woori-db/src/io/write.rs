use chrono::{DateTime, Utc};
use std::io::Error;
use std::{fs::OpenOptions, io::Write};

pub fn write_to_log(log: &str) -> Result<usize, Error> {
    let utc: DateTime<Utc> = Utc::now();
    let date_log = utc.format("%Y_%m_%d.log").to_string();

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(date_log)?;

    let written_bytes = file.write(log.as_bytes())?;

    Ok(written_bytes)
}
pub fn write_to_uniques(log: &str) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("uniques.log")?;

    let _ = file.write(log.as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::io::read::{assert_uniques, assert_content};
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
}