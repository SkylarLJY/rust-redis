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

