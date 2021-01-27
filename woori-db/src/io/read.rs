#[cfg(test)]
use std::{fs::OpenOptions, io::Read};

#[cfg(test)]
pub fn assert_content(pat: &str) {
    let mut file = OpenOptions::new()
            .read(true)
            .open("current.log").unwrap();
    let mut s = String::new();
    file.read_to_string(&mut s).unwrap();

    assert!(s.contains(pat));
}
