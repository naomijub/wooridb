use std::{fs::OpenOptions, io::Write};
use std::io::Error;
use chrono::{DateTime, Utc};

pub async fn write_to_log(log: &str) -> Result<usize, Error> {
    let utc: DateTime<Utc> = Utc::now();
    let date_log = utc.format("%Y_%m_%d.log").to_string();

    let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(date_log)?;
    
    let written_bytes = file.write(log.as_bytes())?;
    
    Ok(written_bytes)
}