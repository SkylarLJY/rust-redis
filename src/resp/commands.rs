use super::{
    datastore::{get_value, set_value},
    errors::UserInputError,
    redisconfig, resp_value::RespType,
};

pub enum RedisCommand {
    Ping,
    Echo,
    Get,
    Set,
    Unknown(String),
    Config,
}

impl RedisCommand {
    pub fn from_str(cmd: &str) -> Self {
        match cmd.to_lowercase().as_str() {
            "ping" => RedisCommand::Ping,
            "echo" => RedisCommand::Echo,
            "get" => RedisCommand::Get,
            "set" => RedisCommand::Set,
            "config" => RedisCommand::Config,
            _ => RedisCommand::Unknown(cmd.to_string()),
        }
    }
}

pub fn handle_input_cmd(input: Vec<&str>) -> Result<RespType, UserInputError> {
    let cmd = input;
    let resp_cmd = RedisCommand::from_str(cmd[0]);
    println!("{:?}", cmd.join(" ").replace("\r\n", "\\r\\n"));
    match resp_cmd {
        RedisCommand::Ping => Ok(RespType::SimpleString("PONG".to_string())),
        RedisCommand::Echo => {
            if cmd.len() > 1 {
                Ok(RespType::SimpleString(cmd[1..].join("")))
            } else {
                Err(UserInputError::InvalidInput(
                    "No message provided to ECHO".to_string(),
                ))
            }
        }
        RedisCommand::Get => {
            if cmd.len() > 1 {
                match get_value(cmd[1]) {
                    Ok(value) => Ok(RespType::SimpleString(value)),
                    Err(e) => Err(UserInputError::DataStoreError(e)),
                }
            } else {
                Err(UserInputError::InvalidInput(
                    "No key provided to GET".to_string(),
                ))
            }
        }
        RedisCommand::Set => {
            if cmd.len() > 2 {
                match set_value(cmd[1].to_string(), cmd[2].to_string()) {
                    None => Ok(RespType::SimpleString("OK".to_string())),
                    Some(e) => Err(UserInputError::DataStoreError(e)),
                }
            } else {
                Err(UserInputError::InvalidInput(
                    "No key/value provided to SET".to_string(),
                ))
            }
        }
        RedisCommand::Config => {
            match cmd.get(1) {
                Some(action) if action.to_lowercase() == "get" => {
                    let key = cmd.get(2).ok_or(UserInputError::InvalidInput(
                        "No key provided for CONFIG GET".to_string(),
                    ))?;
                    match redisconfig::get_config(key) {
                        Some(value) => {
                            let key_resp = RespType::BulkString(Some(key.as_bytes().to_vec()));
                            let val_resp = RespType::BulkString(Some(value.as_bytes().to_vec()));
                            let res_resp = RespType::Array(Some(vec![key_resp, val_resp]));
                            Ok(res_resp)
                        }
                        None => Ok(RespType::Array(None)),
                    }
                }
                Some(action) if action.to_lowercase() == "set" => {
                    let key = cmd.get(2).ok_or(UserInputError::InvalidInput(
                        "No key provided for CONFIG SET".to_string(),
                    ))?;
                    let val = cmd.get(3).ok_or(UserInputError::InvalidInput(
                        "No value provided for CONFIG SET".to_string(),
                    ))?;
                    // TODO: set config value
                    Ok(RespType::SimpleString("OK".to_string()))
                }
                None => Err(UserInputError::InvalidInput(
                    "No action provided for CONFIG command".to_string(),
                )),
                _ => Err(UserInputError::InvalidInput(format!(
                    "Invalid action provided for CONFIG command: {}",
                    cmd[1]
                ))),
            }
        }
        RedisCommand::Unknown(cmd) => Err(UserInputError::UnknownCommand(cmd)),
    }
}
