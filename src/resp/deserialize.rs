use std::str::Lines;

use super::resp_value::RespType;

//TODO: add error handling
pub fn deserialize(input: &str) -> RespType {
    let mut lines = input.lines();
    if let Some(line) = lines.next() {
        deserialize_single(line, &mut lines)
    } else {
        panic!("Empty input");
    }
}

fn deserialize_single(input: &str, lines: &mut Lines) -> RespType {
    match input.chars().next() {
        Some('+') => RespType::SimpleString(input[1..].to_string()),
        Some('-') => RespType::Error(input[1..].to_string()),
        Some(':') => RespType::Integer(input[1..].parse().unwrap()),
        Some('$') => {
            let len = input[1..].parse().unwrap();
            if len == -1 {
                RespType::BulkString(None)
            } else {
                read_bulk_string(len, lines)
            }
        }
        Some('*') => {
            let len = input[1..].parse().unwrap();
            if len == -1 {
                RespType::Array(None)
            } else {
                let mut res = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    if let Some(line) = lines.next() {
                        res.push(deserialize_single(line, lines));
                    }
                }
                RespType::Array(Some(res))
            }
        }
        Some(_) => panic!("Invalid input"),
        None => panic!("Empty input"),
    }
}

fn read_bulk_string(len: i64, lines: &mut Lines) -> RespType {
    // read by line until have enough bytes
    let mut res: Vec<u8> = Vec::with_capacity(len as usize);
    while res.len() < len as usize {
        if let Some(line) = lines.next() {
            res.extend_from_slice(line.as_bytes());
        }
    }

    RespType::BulkString(Some(res))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_simple_string() {
        let input = "+OK\r\n";
        let expected = RespType::SimpleString("OK".to_string());
        assert_eq!(deserialize(input), expected);
    }

    #[test]
    fn test_deserialize_null_array(){
        let input = "*-1\r\n";
        let expected = RespType::Array(None);
        assert_eq!(deserialize(input), expected);
    }

    #[test]
    fn test_deserialize_null_bulk_string(){
        let input = "$-1\r\n";
        let expected = RespType::BulkString(None);
        assert_eq!(deserialize(input), expected);
    }

    #[test]
    fn test_seserialize_bulk_string() {
        let input = "$6\r\nfoobar\r\n";
        let expected = RespType::BulkString(Some("foobar".to_string().into_bytes()));
        assert_eq!(deserialize(input), expected);
    }

    #[test]
    fn test_deserialize_array_simple() {
        let input = "*2\r\n+foo\r\n-bar\r\n";
        let expected = RespType::Array(Some(vec![
            RespType::SimpleString("foo".to_string()),
            RespType::Error("bar".to_string()),

        ]));
        assert_eq!(deserialize(input), expected);
    }

    #[test]
    fn test_deserialize_array_of_bulk_string() {
        let input = "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let expected = RespType::Array(Some(vec![
            RespType::BulkString(Some("foo".to_string().into_bytes())),
            RespType::BulkString(Some("bar".to_string().into_bytes())),
        ]));
        assert_eq!(deserialize(input), expected);
    }

    #[test]
    fn test_deserialize_nested_array(){
        let input = "*2\r\n*2\r\n+foo\r\n-bar\r\n*2\r\n+foo\r\n-bar\r\n";
        let expected = RespType::Array(Some(vec![
            RespType::Array(Some(vec![
                RespType::SimpleString("foo".to_string()),
                RespType::Error("bar".to_string()),
            ])),
            RespType::Array(Some(vec![
                RespType::SimpleString("foo".to_string()),
                RespType::Error("bar".to_string()),
            ]),
            ),
        ]));
        assert_eq!(deserialize(input), expected);
    }
}
