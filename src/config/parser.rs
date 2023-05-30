use crate::config::base::*;

use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Result};

pub fn read_new_config(path: &'static str) -> Result<NewConfig> {
    let reader = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)),
    };

    return match serde_json::from_reader(reader) {
        Ok(config) => Ok(config),
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)),
    };
}
