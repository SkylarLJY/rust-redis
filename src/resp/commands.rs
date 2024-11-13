use super::{
    datastore::{get_value, set_value},
    errors::UserInputError,
    resp_value::RespType,
};
use once_cell::sync::Lazy;
use redis::{Client, Connection};

pub enum RedisCommand {
    Ping,
    Echo(String),
    Get(String),
    Set(String, String, Vec<String>), // key, value, options
    Unknown(String),
    Config(Vec<String>),
}

impl RedisCommand {
    pub fn from_str(cmd: Vec<&str>) -> Self {
        match cmd.get(0).unwrap().to_lowercase().as_str() {
            "ping" => RedisCommand::Ping,
            "echo" => RedisCommand::Echo(cmd.get(1).unwrap_or(&"").to_string()),
            "get" => RedisCommand::Get(cmd.get(1).unwrap_or(&"").to_string()),
            "set" => RedisCommand::Set(
                cmd.get(1).unwrap_or(&"").to_string(),
                cmd.get(2).unwrap_or(&"").to_string(),
                cmd[3..].to_vec().iter().map(|x| x.to_string()).collect(),
            ),
            "config" => {
                RedisCommand::Config(cmd[1..].to_vec().iter().map(|x| x.to_string()).collect())
            }
            _ => RedisCommand::Unknown(cmd.join(" ")),
        }
    }
}

pub fn handle_input_cmd(cmd: Vec<&str>) -> Result<RespType, UserInputError> {
    let resp_cmd = RedisCommand::from_str(cmd);
    // println!("{:?}", cmd.join(" ").replace("\r\n", "\\r\\n"));

    match resp_cmd {
        RedisCommand::Ping => Ok(RespType::SimpleString("PONG".to_string())),
        RedisCommand::Echo(s) => {
            if s.len() > 0 {
                Ok(RespType::SimpleString(s))
            } else {
                Err(UserInputError::InvalidInput(
                    "No message provided to ECHO".to_string(),
                ))
            }
        }
        RedisCommand::Get(key) => {
            if key.len() > 0 {
                match get_value(&key) {
                    Ok(value) => Ok(RespType::SimpleString(value)),
                    Err(e) => Err(UserInputError::DataStoreError(e)),
                }
            } else {
                Err(UserInputError::InvalidInput(
                    "No key provided to GET".to_string(),
                ))
            }
        }
        RedisCommand::Set(key, value, options) => {
            if key.len() > 0 && value.len() > 0 {
                match set_value(key, value, options) {
                    Ok(resp) => Ok(resp),
                    Err(e) => Err(UserInputError::DataStoreError(e)),
                }
            } else {
                Err(UserInputError::InvalidInput(
                    "No key/value provided to SET".to_string(),
                ))
            }
        }
        RedisCommand::Config(ops) => {
            Ok(RespType::Error("Unimplemented".to_string()))
            //     match ops.get(0) {
            //         Some(action) if action.to_lowercase() == "get" => {
            //             let key = ops.get(1).ok_or(UserInputError::InvalidInput(
            //                 "No key provided for CONFIG GET".to_string(),
            //             ))?;
            //             match redisconfig::get_config(key) {
            //                 Some(value) => {
            //                     let key_resp = RespType::BulkString(Some(Bytes::from(key.as_bytes())));
            //                     let val_resp = RespType::BulkString(Some(Bytes::from(value)));
            //                     let res_resp = RespType::Array(Some(vec![key_resp, val_resp]));
            //                     Ok(res_resp)
            //                 }
            //                 None => Ok(RespType::Array(None)),
            //             }
            //         }
            //         Some(action) if action.to_lowercase() == "set" => {
            //             let key = cmd.get(2).ok_or(UserInputError::InvalidInput(
            //                 "No key provided for CONFIG SET".to_string(),
            //             ))?;
            //             let val = cmd.get(3).ok_or(UserInputError::InvalidInput(
            //                 "No value provided for CONFIG SET".to_string(),
            //             ))?;
            //             // TODO: set config value
            //             Ok(RespType::SimpleString("OK".to_string()))
            //         }
            //         None => Err(UserInputError::InvalidInput(
            //             "No action provided for CONFIG command".to_string(),
            //         )),
            //         _ => Err(UserInputError::InvalidInput(format!(
            //             "Invalid action provided for CONFIG command: {}",
            //             cmd[1]
            //         ))),
            //     }
        }
        RedisCommand::Unknown(cmd) => Err(UserInputError::UnknownCommand(cmd)),
    }
}

// ===== tests =====

static mut REDIS_CONN: Lazy<Connection> = Lazy::new(|| {
    let client = Client::open("redis://127.0.0.1/").expect("Failed to connect to redis");
    client
        .get_connection()
        .expect("Failed to get redis connection")
});

#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use redis::Value;

    use super::*;

    fn redis_value_to_string(val: &Value) -> String {
        match val {
            Value::SimpleString(s) => s.to_string(),
            Value::BulkString(s) => from_utf8(s.as_slice())
                .expect("Failed to convert bulk string to string")
                .to_string(),
            Value::Array(arr) => {
                let mut res = String::new();
                for v in arr.iter() {
                    res.push_str(&redis_value_to_string(v));
                    res.push_str(" ");
                }
                res
            }
            _ => panic!("Invalid response from redis"),
        }
    }

    #[test]
    #[ignore]
    fn test_get_config() {
        let mut cmd = redis::cmd("CONFIG");
        cmd.arg("GET").arg("save");
        let redis_res: Value = cmd.query(unsafe { &mut REDIS_CONN }).expect("ERR");

        let rust_redis_res = handle_input_cmd(vec!["config", "get", "save"])
            .expect("Failed to get config from rust redis");
        match (redis_res, rust_redis_res) {
            (Value::Array(arr1), RespType::Array(arr2)) => {
                let key = redis_value_to_string(&arr1[0]);
                let val = redis_value_to_string(&arr1[1]);
                let arr2 = arr2.unwrap();
                let rust_key = match &arr2[0] {
                    RespType::BulkString(Some(bs)) => std::str::from_utf8(bs).unwrap(),
                    _ => panic!("Invalid response from rust redis"),
                };
                let rust_val = match &arr2[1] {
                    RespType::BulkString(Some(bs)) => std::str::from_utf8(bs).unwrap(),
                    _ => panic!("Invalid response from rust redis"),
                };
                assert_eq!(key, rust_key);
                assert_eq!(val, rust_val);
            }
            _ => panic!("Invalid response from redis"),
        };
    }

    #[test]
    fn test_set_with_expire() {}
}
