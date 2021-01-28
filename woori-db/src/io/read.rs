#[cfg(test)]
use std::{fs::OpenOptions, io::Read};

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
