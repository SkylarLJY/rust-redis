use super::constants::{NULL_ARRAY, NULL_BULK_STRING};

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

    pub fn get_byte_length(&self) -> usize {
        match self {
            RespType::SimpleString(s) => s.len() + 3,
            RespType::Error(e) => e.len() + 3,
            RespType::Integer(i) => i.to_string().len() + 3,
            RespType::BulkString(b) => match b {
                None => NULL_BULK_STRING.len(), 
                Some(b) => b.len() + 5 + b.len().to_string().len(), // ${len}\r\n{content}\r\n
            },
            RespType::Array(a) => match a {
                None => NULL_ARRAY.len(), 
                Some(a) => {
                    let mut res = 3 + a.len().to_string().len(); // *{len}\r\n
                    for e in a {
                        res += e.get_byte_length();
                    }
                    res
                }
            }
        }
    }
}
