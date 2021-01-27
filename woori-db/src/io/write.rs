use std::{fs::OpenOptions, io::Write};
use std::io::Error;

pub async fn write_to_log(log: &str) -> Result<usize, Error> {
    let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("current.log")?;
    
    let written_bytes = file.write(log.as_bytes())?;
    
    Ok(written_bytes)
}