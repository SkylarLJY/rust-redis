use lazy_static::lazy_static;
use std::{collections::HashMap, time::Duration};
use parking_lot::Mutex;

use crate::resp::errors::DataStoreError;

lazy_static! {
   static ref DATA: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
   // define some constants
    static ref OP_TIMEOUT_SECS: Duration = Duration::from_secs(1);
}

pub fn get_value(key: &str) -> Result<String, DataStoreError> {
    let data = DATA.try_lock_for(*OP_TIMEOUT_SECS);
    match data {
        None => {
            eprintln!("Failed to acquire lock to get value");
            Err(DataStoreError::LockError)
        },
        Some(data) => {
            match data.get(key) {
                Some(value) => Ok(value.clone()),
                None => Err(DataStoreError::KeyNotFound),
            }
        },
    }
}

pub fn set_value(key: String, value: String) -> Option<DataStoreError> {
    let data = DATA.try_lock_for(*OP_TIMEOUT_SECS);
    if data.is_none() {
        eprintln!("Failed to acquire lock to set value");
        return Some(DataStoreError::LockError);
    }

    match data {
        None => {
            eprintln!("Failed to acquire lock to set value");
            Some(DataStoreError::LockError)
        },
        Some(mut d) => {
            d.insert(key, value);
            None
        }
    }
}