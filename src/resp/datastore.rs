use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::io::Read;
use std::{collections::HashMap, fs::OpenOptions, io::Write, time::Duration};

use crate::resp::constants::DATA_FILE_PATH;
use crate::resp::errors::DataStoreError;

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct MapValue {
    value: String,
    expire_at: Option<i64>,
}

lazy_static! {
    static ref DATA: Mutex<HashMap<String, MapValue>> = Mutex::new(HashMap::new());
    static ref OP_TIMEOUT_SECS: Duration = Duration::from_secs(1);
}

pub fn get_value(key: &str) -> Result<String, DataStoreError> {
    let data = DATA.try_lock_for(*OP_TIMEOUT_SECS);
    let now = Utc::now().timestamp();
    match data {
        None => {
            eprintln!("Failed to acquire lock to get value");
            Err(DataStoreError::LockError)
        }
        Some(mut data) => match data.get(key) {
            Some(v) if v.expire_at.is_none() => Ok(v.value.clone()),
            Some(v) if v.expire_at.is_some() && v.expire_at.unwrap() > now => Ok(v.value.clone()),
            Some(v) => {
                data.remove(key);
                Err(DataStoreError::ExpiredKey)
            }
            None => Err(DataStoreError::KeyNotFound),
        },
    }
}

pub fn set_value(key: String, value: String, options: Vec<&str>) -> Option<DataStoreError> {
    let data = DATA.try_lock_for(*OP_TIMEOUT_SECS);
    if data.is_none() {
        eprintln!("Failed to acquire lock to set value");
        return Some(DataStoreError::LockError);
    }

    // parse options
    let exp = options
        .iter()
        .enumerate()
        .find_map(|(i, x)| match x.to_uppercase().as_str() {
            "EX" => {
                if options.len() <= i + 1 {
                    eprintln!("No expire time provided");
                    return Some(Err(DataStoreError::InvalidInput(
                        "No expire time provided".to_string(),
                    )));
                }
                let duration = options[i + 1].parse::<u64>().unwrap();
                let expire_date_time = Utc::now() + chrono::Duration::seconds(duration as i64);
                Some(Ok(expire_date_time.timestamp()))
            }
            "PX" => {
                if options.len() <= i + 1 {
                    eprintln!("No expire time provided");
                    return Some(Err(DataStoreError::InvalidInput(
                        "No expire time provided".to_string(),
                    )));
                }
                let duration = options[i + 1].parse::<u64>().unwrap();
                let expire_date_time = Utc::now() + chrono::Duration::milliseconds(duration as i64);
                Some(Ok(expire_date_time.timestamp()))
            }
            "EXAT" => {
                if options.len() <= i + 1 {
                    eprintln!("No expire time provided");
                    return Some(Err(DataStoreError::InvalidInput(
                        "No expire time provided".to_string(),
                    )));
                }
                let expire_date_time = options[i + 1].parse::<i64>().unwrap();
                Some(Ok(expire_date_time))
            }
            "PXAT" => {
                if options.len() <= i + 1 {
                    eprintln!("No expire time provided");
                    return Some(Err(DataStoreError::InvalidInput(
                        "No expire time provided".to_string(),
                    )));
                }
                let expire_date_time = options[i + 1].parse::<i64>().unwrap();
                Some(Ok(expire_date_time))
            }
            _ => None,
        });

    let expire_at = match exp {
        Some(Ok(exp_date_time)) => Some(exp_date_time),
        Some(Err(e)) => return Some(e),
        None => None,
    };

    match data {
        None => {
            eprintln!("Failed to acquire lock to set value");
            Some(DataStoreError::LockError)
        }
        Some(mut d) => {
            d.insert(key, MapValue { value, expire_at });
            None
        }
    }
}

pub fn save() -> Result<(), DataStoreError> {
    let data = DATA.try_lock_for(*OP_TIMEOUT_SECS);
    match data {
        None => {
            eprintln!("Failed to acquire lock to save data");
            Err(DataStoreError::LockError)
        }
        Some(data) => {
            let json_data =
                serde_json::to_string(&*data).map_err(|_| DataStoreError::SerializeError)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(DATA_FILE_PATH)
                .map_err(|_| DataStoreError::FileIOError)?;
            file.write_all(json_data.as_bytes())
                .map_err(|_| DataStoreError::FileIOError)?;
            Ok(())
        }
    }
}

pub fn load() -> Result<(), DataStoreError> {
    let mut file = OpenOptions::new()
        .read(true)
        .open(DATA_FILE_PATH)
        .map_err(|_| DataStoreError::FileIOError)?;
    let mut json_str = String::new();
    file.read_to_string(&mut json_str)
        .map_err(|_| DataStoreError::FileIOError)?;
    let map: HashMap<String, MapValue> =
        serde_json::from_str(json_str.as_str()).map_err(|_| DataStoreError::DataLoadError)?;
    let mut data = DATA
        .try_lock_for(*OP_TIMEOUT_SECS)
        .ok_or(DataStoreError::LockError)?;
    *data = map;
    Ok(())
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;

    fn clear_data() {
        DATA.lock().clear();
    }

    #[test]
    #[serial]
    fn test_set_value() {
        clear_data();
        let key = "key".to_string();
        let value = "value".to_string();

        let result = set_value(key.clone(), value.clone(), vec![]);
        assert_eq!(result, None);
    }

    #[test]
    #[serial]
    fn test_get_value() {
        clear_data();
        let key = "key".to_string();
        let value = "value".to_string();
        let _ = set_value(key.clone(), value.clone(), vec![]);

        let result = get_value(&key);
        assert_eq!(result, Ok(value));
    }

    #[test]
    #[serial]
    fn test_get_value_key_not_found() {
        clear_data();
        let key = "key".to_string();

        let result = get_value(&key);
        assert_eq!(result, Err(DataStoreError::KeyNotFound));
    }

    #[test]
    #[serial]
    fn test_set_value_lock_error() {
        clear_data();
        let key = "key".to_string();
        let value = "value".to_string();

        let lock = DATA.lock();
        let result = set_value(key.clone(), value.clone(), vec![]);
        assert_eq!(result, Some(DataStoreError::LockError));
    }

    #[test]
    #[serial]
    fn test_get_expired_key() {
        clear_data();
        let key = "key".to_string();
        let value = "value".to_string();
        let result = set_value(key.clone(), value.clone(), vec!["EX", "1"]);
        assert!(result.is_none());
        // wait for key to expire
        std::thread::sleep(Duration::from_secs(2));
        let result = get_value(&key);
        assert_eq!(result, Err(DataStoreError::ExpiredKey));
    }
}
