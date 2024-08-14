#[derive(Debug, PartialEq)]
pub enum RespType {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Option<Vec<RespType>>),
}

impl RespType {
    pub fn serialize(&self) -> String {
        match self {
            RespType::SimpleString(s) => format!("+{}\r\n", s),
            RespType::Error(e) => format!("-{}\r\n", e),
            RespType::Integer(i) => format!(":{}\r\n", i),
            RespType::BulkString(b) => match b {
                None => "$-1\r\n".to_string(),
                Some(b) => format!("${}\r\n{}\r\n", b.len(), String::from_utf8_lossy(b)),
            },
            RespType::Array(a) => match a {
                None => "*-1\r\n".to_string(),
                Some(a) => {
                    let mut res = format!("*{}\r\n", a.len());
                    for e in a {
                        res.push_str(&e.serialize());
                    }
                    res
                }
            }

        }
    }
}