pub enum RedisCommand {
    Ping,
    Echo,
    Unknown,
}

impl RedisCommand {
    pub fn from_str(cmd: &str) -> Self {
        match cmd.to_lowercase().as_str() {
            "ping" => RedisCommand::Ping,
            "echo" => RedisCommand::Echo,
            _ => RedisCommand::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            RedisCommand::Ping => "PING",
            RedisCommand::Echo => "ECHO",
            RedisCommand::Unknown => "UNKNOWN",
        }
    }
}

