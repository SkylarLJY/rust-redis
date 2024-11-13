use bytes::Bytes;

use super::constants::*;
use super::errors::DeserializeError;
use super::resp_value::RespType;

// return (RespType, number of bytes consumed)
pub fn deserialize(input: &[u8]) -> Result<(RespType, usize), DeserializeError> {
    let res;
    match input.get(0) {
        Some(&SIMPLE_STRING_PREFIX) => {
            let end_idx = input.iter().position(|&c| c == b'\r').unwrap();
            res = Ok(RespType::SimpleString(u8_to_string(&input[1..end_idx])));
        }
        Some(&ERROR_PREFIX) => {
            let end_idx = input.iter().position(|&c| c == b'\r').unwrap();
            res = Ok(RespType::Error(u8_to_string(&input[1..end_idx])));
        }
        Some(&INTEGER_PREFIX) => {
            let end_idx = input.iter().position(|&c| c == b'\r').unwrap();
            let num = u8_to_string(&input[1..end_idx]).parse().unwrap();
            res = Ok(RespType::Integer(num));
        }
        Some(&BULK_STRING_PREFIX) => {
            res = deserialize_bulk_string(input);
        }
        Some(&ARRAY_PREFIX) => {
            res = deserialize_array(input);
        }
        None => return Err(DeserializeError::EmptyInput),
        Some(_) => {
            return Err(DeserializeError::InvalidInput(format!(
                "Invalid input: {:?}",
                input
            )))
        }
    }
    if res.is_err() {
        return Err(res.err().unwrap());
    } else {
        let res = res.unwrap();
        let len = res.get_byte_length();
        Ok((res, len))
    }
}

pub fn deserialize_bulk_string(input: &[u8]) -> Result<RespType, DeserializeError> {
    if input.len() < 4 || input[0] != BULK_STRING_PREFIX {
        return Err(DeserializeError::InvalidInput(format!(
            "Invalid input: {:?}",
            input
        )));
    }
    if input.len() == NULL_BULK_STRING.len() && input[1] == b'-' && input[2] == b'1' {
        return Ok(RespType::BulkString(None));
    }
    let (len, content_start_idx) = match get_length_and_content_start_idx(input) {
        Ok((len, content_start_idx)) => (len, content_start_idx),
        Err(e) => return Err(e),
    };

    let content = input[content_start_idx..content_start_idx + len].to_vec();
    // check if content is at the end and followed by CRLF
    if input[content_start_idx + len] != b'\r' || input[content_start_idx + len + 1] != b'\n' {
        return Err(DeserializeError::LengthMismatch(
            "Bulk string length does not match content length".to_string(),
        ));
    }

    Ok(RespType::BulkString(Some(Bytes::from(content))))
}

pub fn deserialize_array(input: &[u8]) -> Result<RespType, DeserializeError> {
    if input.len() < 4 || input[0] != ARRAY_PREFIX {
        return Err(DeserializeError::InvalidInput(format!(
            "Invalid input: {:?}",
            input
        )));
    }
    if input.len() == NULL_ARRAY.len() && input[1] == b'-' && input[2] == b'1' {
        return Ok(RespType::Array(None));
    }
    let (len, mut content_start_idx) = match get_length_and_content_start_idx(&input) {
        Ok((len, content_start_idx)) => (len, content_start_idx),
        Err(e) => return Err(e),
    };

    // for now parse all elements at once. May switch to lazy parsing in the future
    let mut content: Vec<RespType> = Vec::with_capacity(len);
    for _ in 0..len {
        match deserialize(&input[content_start_idx..]) {
            Err(e) => {
                match e {
                    DeserializeError::EmptyInput => {
                        return Err(DeserializeError::InvalidInput(
                            "Array length value larger than actual length".to_string(),
                        ));
                    }
                    _ => return Err(e),
                };
            }
            Ok((elem, bytes_consumed)) => {
                content.push(elem);
                content_start_idx += bytes_consumed;
            }
        }
    }
    Ok(RespType::Array(Some(content)))
}

fn get_length_and_content_start_idx(input: &[u8]) -> Result<(usize, usize), DeserializeError> {
    let crlf_idx = match input.iter().position(|&c| c == b'\r') {
        Some(idx) => idx,
        None => {
            return Err(DeserializeError::InvalidInput(format!(
                "Failed to find crlf in input: {:?}",
                input
            )))
        }
    };
    let len_str = match std::str::from_utf8(&input[1..crlf_idx]) {
        Ok(s) => s,
        Err(_) => {
            return Err(DeserializeError::InvalidInput(format!(
                "Failed to parse length string into utf8 string: {:?}",
                &input[1..crlf_idx]
            )))
        }
    };
    let len: usize = match len_str.parse() {
        Ok(n) => n,
        Err(_) => {
            return Err(DeserializeError::InvalidInput(format!(
                "Failed to parse length string into integer: {:?}",
                len_str
            )))
        }
    };
    Ok((len, crlf_idx + 2))
}

fn u8_to_string(input: &[u8]) -> String {
    match std::str::from_utf8(input) {
        Ok(s) => s.to_string(),
        Err(_) => panic!("Failed to convert input to string"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod valid_inputs {
        use super::*;
        #[test]
        fn test_deserialize_simple_string() {
            let input = b"+OK\r\n";
            let expected = RespType::SimpleString("OK".to_string());
            assert_eq!(deserialize(input).unwrap(), (expected, input.len()));
        }

        #[test]
        fn test_deserialize_null_array() {
            let input = NULL_ARRAY;
            let expected = RespType::Array(None);
            assert_eq!(deserialize(input).unwrap(), (expected, input.len()));
        }

        #[test]
        fn test_deserialize_null_bulk_string() {
            let input = NULL_BULK_STRING;
            let expected = RespType::BulkString(None);
            assert_eq!(deserialize(input).unwrap(), (expected, input.len()));
        }

        #[test]
        fn test_deserialize_array_simple() {
            let input = b"*2\r\n+foo\r\n-bar\r\n";
            let expected = RespType::Array(Some(vec![
                RespType::SimpleString("foo".to_string()),
                RespType::Error("bar".to_string()),
            ]));
            assert_eq!(deserialize(input).unwrap(), (expected, input.len()));
        }

        #[test]
        fn test_deserialize_array_of_bulk_string() {
            let input = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
            let expected = RespType::Array(Some(vec![
                RespType::BulkString(Some(Bytes::from("foo"))),
                RespType::BulkString(Some(Bytes::from("bar"))),
            ]));
            assert_eq!(deserialize(input).unwrap(), (expected, input.len()));
        }

        #[test]
        fn test_deserialize_nested_array() {
            let input = b"*2\r\n*2\r\n+foo\r\n-bar\r\n*2\r\n+foo\r\n-bar\r\n";
            let expected = RespType::Array(Some(vec![
                RespType::Array(Some(vec![
                    RespType::SimpleString("foo".to_string()),
                    RespType::Error("bar".to_string()),
                ])),
                RespType::Array(Some(vec![
                    RespType::SimpleString("foo".to_string()),
                    RespType::Error("bar".to_string()),
                ])),
            ]));
            assert_eq!(deserialize(input).unwrap(), (expected, input.len()));
        }

        #[test]
        fn test_deserialize_bulk_string_with_newline() {
            let input = b"$8\r\nfoo\r\nbar\r\n";
            let expected = RespType::BulkString(Some(Bytes::from("foo\r\nbar")));
            assert_eq!(deserialize(input).unwrap(), (expected, input.len()));
        }

        #[test]
        fn test_deserialize_mixed_array() {
            let input = b"*3\r\n+foo\r\n$3\r\nbar\r\n:42\r\n";
            let expected = RespType::Array(Some(vec![
                RespType::SimpleString("foo".to_string()),
                RespType::BulkString(Some(Bytes::from("bar"))),
                RespType::Integer(42),
            ]));
            assert_eq!(deserialize(input).unwrap(), (expected, input.len()));
        }
    }

    mod error_cases {
        use super::*;

        #[test]
        fn test_deserialize_bulk_string_wrong_length() {
            let input = b"$3\r\nfooooooo\r\n";
            let res = deserialize(input);
            assert!(res.is_err());
            assert_eq!(
                res.err().unwrap(),
                DeserializeError::LengthMismatch(
                    "Bulk string length does not match content length".to_string()
                )
            );
        }
        #[test]
        fn test_deserialize_invalid_input() {
            let input = b"!foo\r\n";
            let res = deserialize(input);
            assert!(res.is_err());
            assert_eq!(
                res.err().unwrap(),
                DeserializeError::InvalidInput(format!("Invalid input: {:?}", input))
            );
        }

        #[test]
        fn test_deserialize_array_actual_length_too_short() {
            let input = b"*5\r\n+foo\r\n-bar\r\n+foo\r\n";
            let res = deserialize(input);
            assert!(res.is_err());
            match res.err().unwrap() {
                DeserializeError::InvalidInput(s) => {
                    assert_eq!(s, "Array length value larger than actual length")
                }
                _ => panic!("Unexpected error type"),
            }
        }
    }
}
