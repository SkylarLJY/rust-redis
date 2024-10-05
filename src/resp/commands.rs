use super::{datastore::{get_value, set_value}, errors::UserInputError};

pub enum RedisCommand {
    Ping,
    Echo,
    Get,
    Set,
    Unknown,
}

impl RedisCommand {
    pub fn from_str(cmd: &str) -> Self {
        match cmd.to_lowercase().as_str() {
            "ping" => RedisCommand::Ping,
            "echo" => RedisCommand::Echo,
            "get" => RedisCommand::Get,
            "set" => RedisCommand::Set,
            _ => RedisCommand::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RedisCommand::Ping => "PING",
            RedisCommand::Echo => "ECHO",
            RedisCommand::Get => "GET",
            RedisCommand::Set => "SET",
            RedisCommand::Unknown => "UNKNOWN",
        }
    }
}

pub fn handle_input_cmd(input: Vec<&str>) -> Result<String, UserInputError> {
    let cmd = input;

    let resp_cmd = RedisCommand::from_str(cmd[0]);
    match resp_cmd {
        RedisCommand::Ping => {
            Ok("PONG".to_string())
        },
        RedisCommand::Echo => {
            if cmd.len() > 1 {
                Ok(cmd[1..].join(""))
            } else {
                Err(UserInputError::InvalidInput("No message provided to ECHO".to_string()))
            }
        },
        RedisCommand::Get => {
            if cmd.len() > 1 {
                match get_value(cmd[1]) {
                    Ok(value) => Ok(value),
                    Err(e) => Err(UserInputError::DataStoreError(e)),
                }
            } else {
                Err(UserInputError::InvalidInput("No key provided to GET".to_string()))
            }
        },
        RedisCommand::Set => {
            if cmd.len() > 2 {
                match set_value(cmd[1].to_string(), cmd[2].to_string()) {
                    None => Ok("OK".to_string()),
                    Some(e) => Err(UserInputError::DataStoreError(e)),
                }
            } else {
                Err(UserInputError::InvalidInput("No key/value provided to SET".to_string()))
            }
        },
        _ => {
            Err(UserInputError::UnknownCommand)
        }
    }
}

