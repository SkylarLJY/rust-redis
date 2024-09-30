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
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            RespType::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
            RespType::Error(e) => format!("-{}\r\n", e).into_bytes(),
            RespType::Integer(i) => format!(":{}\r\n", i).into_bytes(),
            RespType::BulkString(b) => match b {
                None => NULL_BULK_STRING.to_vec(),
                Some(b) => {
                    let mut res = format!("${}\r\n", b.len()).into_bytes();
                    res.extend_from_slice(b);
                    res.extend_from_slice(b"\r\n");
                    res
                }
            },
            RespType::Array(a) => match a {
                None => NULL_ARRAY.to_vec(),
                Some(a) => {
                    let mut res = format!("*{}\r\n", a.len()).into_bytes();
                    for e in a {
                        res.extend_from_slice(&e.serialize());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple_string() {
        let input = RespType::SimpleString("OK".to_string());
        let expected = b"+OK\r\n";
        assert_eq!(input.serialize(), expected);
    }

    #[test]
    fn test_serialize_null_array() {
        let input = RespType::Array(None);
        let expected = NULL_ARRAY;
        assert_eq!(input.serialize(), expected);
    }

    #[test]
    fn test_serialize_null_bulk_string() {
        let input = RespType::BulkString(None);
        let expected = NULL_BULK_STRING;
        assert_eq!(input.serialize(), expected);
    }

    #[test]
    fn test_serialize_array_simple() {
        let input = RespType::Array(Some(vec![
            RespType::SimpleString("foo".to_string()),
            RespType::Error("bar".to_string()),
        ]));
        let expected = b"*2\r\n+foo\r\n-bar\r\n";
        assert_eq!(input.serialize(), expected);
    }

    #[test]
    fn test_serialize_array_of_bulk_string() {
        let input = RespType::Array(Some(vec![
            RespType::BulkString(Some("foo".to_string().into_bytes())),
            RespType::BulkString(Some("bar".to_string().into_bytes())),
        ]));
        let expected = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        assert_eq!(input.serialize(), expected);
    }
}
