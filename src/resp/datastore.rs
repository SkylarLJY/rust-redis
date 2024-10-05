use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::{collections::HashMap, time::Duration};

use crate::resp::errors::DataStoreError;

lazy_static! {
    static ref DATA: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref OP_TIMEOUT_SECS: Duration = Duration::from_secs(1);
}

pub fn get_value(key: &str) -> Result<String, DataStoreError> {
    let data = DATA.try_lock_for(*OP_TIMEOUT_SECS);
    match data {
        None => {
            eprintln!("Failed to acquire lock to get value");
            Err(DataStoreError::LockError)
        }
        Some(data) => match data.get(key) {
            Some(value) => Ok(value.clone()),
            None => Err(DataStoreError::KeyNotFound),
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
        }
        Some(mut d) => {
            d.insert(key, value);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_data() {
        DATA.lock().clear();
    }

    #[test]
    fn test_set_value() {
        clear_data();
        let key = "key".to_string();
        let value = "value".to_string();

        let result = set_value(key.clone(), value.clone());
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_value() {
        clear_data();
        let key = "key".to_string();
        let value = "value".to_string();
        let _ = set_value(key.clone(), value.clone());

        let result = get_value(&key);
        assert_eq!(result, Ok(value));
    }

    #[test]
    fn test_get_value_key_not_found() {
        clear_data();
        let key = "key".to_string();

        let result = get_value(&key);
        assert_eq!(result, Err(DataStoreError::KeyNotFound));
    }

    #[test]
    fn test_set_value_lock_error() {
        clear_data();
        let key = "key".to_string();
        let value = "value".to_string();

        let lock = DATA.lock();
        let result = set_value(key.clone(), value.clone());
        assert_eq!(result, Some(DataStoreError::LockError));
    }
}
