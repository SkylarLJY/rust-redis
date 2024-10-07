use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use super::{constants::CONFIG_FILE_PATH, errors::ServerError};

// TODO: parse config file 
lazy_static! {
    static ref REDIS_CONFIG: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert("port".to_string(), "6379".to_string());
        m.insert("save".to_string(), "".to_string());
        m.insert("appendonly".to_string(), "no".to_string());
        m
    };
}

pub fn parse_config_file() -> Option<ServerError> {
    let file = File::open(CONFIG_FILE_PATH).ok()?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.ok()?;
        if line.starts_with('#') {
            continue;
        }
        let (k, v) = line.split_once(" ").unwrap();
        // unsafe { REDIS_CONFIG.insert(k.to_string(), v.to_string()) };
    }
    None
}

pub fn get_config(key: &str) -> Option<String> {
    unsafe { REDIS_CONFIG.get(key).cloned() }
}

