use bytes::Bytes;
use chrono::Utc;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::hash::Hash;
use std::io::Read;
use std::sync::Arc;
use std::{collections::HashMap, fs::OpenOptions, io::Write, time::Duration};

use crate::resp::constants::DATA_FILE_PATH;
use crate::resp::errors::DataStoreError;

use serde_derive::{Deserialize, Serialize};

use super::resp_value::RespType;

#[derive(Serialize, Deserialize)]
pub struct MapValue {
    pub value: String,
    expire_at: Option<i64>,
}

type Shard = Mutex<HashMap<String, String>>;

pub struct Db {
    pub data: Arc<Vec<Shard>>,
}

impl Db {
    pub fn new(num_shards: usize) -> Self {
        let mut db_with_shards = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            db_with_shards.push(Mutex::new(HashMap::new()));
        }
        Self {
            data: Arc::new(db_with_shards),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }

    fn get_shard_for_key(&self, key: &str) -> &Shard {
        let i = key.len() % self.data.len();
        &self.data[i]
    }

    pub fn get(&self, key: &str) -> Result<String, DataStoreError> {
        let data = self.get_shard_for_key(key).lock();
        data.get(key)
            .map(|x| x.clone())
            .ok_or(DataStoreError::KeyNotFound)
    }

    pub fn set(&self, key: &str, val: &str, ops: Vec<String>) -> Result<(), DataStoreError> {
        let mut data = self.get_shard_for_key(key).lock();
        data.insert(key.to_string(), val.to_string());
        Ok(())
    }

    // TOD: save & load with .rdb file
    pub fn save(&self) -> Result<(), DataStoreError> {
        let mut json_data = String::new();
        for shard in self.data.iter() {
            let data = shard.lock();
            let conetent =
                serde_json::to_string(&*data).map_err(|_| DataStoreError::SerializeError)?;
            json_data.push_str(conetent.as_str());
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(DATA_FILE_PATH)
            .map_err(|_| DataStoreError::FileIOError)?;
        file.write_all(json_data.as_bytes())
            .map_err(|_| DataStoreError::FileIOError)?;
        Ok(())
    }

    pub fn load(&self) -> Result<(), DataStoreError> {
        todo!()
    }
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

pub fn set_value(
    key: String,
    value: String,
    options: Vec<String>,
) -> Result<RespType, DataStoreError> {
    let data = DATA.try_lock_for(*OP_TIMEOUT_SECS);
    if data.is_none() {
        eprintln!("Failed to acquire lock to set value");
        return Err(DataStoreError::LockError);
    }

    let mut ttl = None;
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
                ttl = Some(duration);
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
                ttl = Some(duration);
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
        Some(Err(e)) => return Err(e),
        None => None,
    };

    let nx = options.iter().find(|x| x.to_uppercase() == "NX");
    let xx = options.iter().find(|x| x.to_uppercase() == "XX");
    let keepttl = options.iter().find(|x| x.to_uppercase() == "KEEPTTL");
    let get = options.iter().find(|x| x.to_uppercase() == "GET");

    match data {
        None => {
            eprintln!("Failed to acquire lock to set value");
            Err(DataStoreError::LockError)
        }
        Some(mut d) => {
            // TODO: handle keepttl
            let mut final_res = RespType::SimpleString("OK".to_string());
            if nx.is_some() && xx.is_some() {
                return Ok(RespType::Error("Syntax error".to_string()));
            }
            // nx - set if not already exists
            // xx - only set if already exists
            if (nx.is_some() && d.contains_key(key.as_str()))
                || (xx.is_some() && !d.contains_key(key.as_str()))
            {
                return Ok(RespType::Null);
            }
            if get.is_some() {
                final_res = match d.get(key.as_str()) {
                    Some(v) => RespType::BulkString(Some(Bytes::from(v.value.clone()))),
                    None => RespType::Null,
                }
            }
            d.insert(key, MapValue { value, expire_at });
            Ok(final_res)
        }
    }
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
        assert_eq!(result, Ok(RespType::SimpleString("OK".to_string())));
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
        assert_eq!(result, Err(DataStoreError::LockError));
    }

    #[test]
    #[serial]
    fn test_get_expired_key() {
        clear_data();
        let key = "key".to_string();
        let value = "value".to_string();
        let result = set_value(
            key.clone(),
            value.clone(),
            vec!["EX".to_string(), "1".to_string()],
        );
        assert!(result.is_ok());
        // wait for key to expire
        std::thread::sleep(Duration::from_secs(2));
        let result = get_value(&key);
        assert_eq!(result, Err(DataStoreError::ExpiredKey));
    }
}
