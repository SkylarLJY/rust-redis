use super::resp_value::RespType;
use super::constants::*;

//TODO: add error handling
// return (RespType, number of bytes consumed)
pub fn deserialize(input: &[u8]) -> (RespType, usize) {
    let res;
    match input.get(0) {
        Some(&SIMPLE_STRING_PREFIX) => {
            let end_idx = input.iter().position(|&c| c == b'\r').unwrap();
            res = RespType::SimpleString(u8_to_string(&input[1..end_idx]));
        },
        Some(&ERROR_PREFIX) => {
            let end_idx = input.iter().position(|&c| c == b'\r').unwrap();
            res = RespType::Error(u8_to_string(&input[1..end_idx]));
        },
        Some(&INTEGER_PREFIX) => {
            let end_idx = input.iter().position(|&c| c == b'\r').unwrap();
            let num = u8_to_string(&input[1..end_idx]).parse().unwrap();
            res = RespType::Integer(num);
        },
        Some(&BULK_STRING_PREFIX) => {
            res = deserialize_bulk_string(input);
        },
        Some(&ARRAY_PREFIX) => {
            res = deserialize_array(input);
        },
        None => panic!("Empty input"),
        Some(_) => panic!("Invalid input"),
    }
    let len = res.get_byte_length();
    (res, len)
}

pub fn deserialize_bulk_string(input: &[u8]) -> RespType {
    // validate input
    if input.len() < 4 || input[0] != BULK_STRING_PREFIX {
        panic!("Invalid input");
    }
    if input.len() == NULL_BULK_STRING.len() && input[1] == b'-' && input[2] == b'1' {
        return RespType::BulkString(None);
    }
    let (len, content_start_idx) = get_length_and_content_start_idx(input);
    let content  = &input[content_start_idx..content_start_idx + len];
    RespType::BulkString(Some(content.to_vec()))
}

pub fn deserialize_array(input: &[u8]) -> RespType {
    // validate input
    if input.len() < 4 || input[0] != ARRAY_PREFIX {
        panic!("Invalid input");
    }
    if input.len() == NULL_ARRAY.len() && input[1] == b'-' && input[2] == b'1' {
        return RespType::Array(None);
    }
    let (len, mut content_start_idx) = get_length_and_content_start_idx(&input);

    // for now parse all elements at once. May switch to lazy parsing in the future
    let mut content: Vec<RespType> = Vec::with_capacity(len);
    for _ in 0..len {
        let (elem, bytes_consumed) = deserialize(&input[content_start_idx..]);
        content.push(elem);
        content_start_idx += bytes_consumed;
    }
    RespType::Array(Some(content))
}

fn get_length_and_content_start_idx(input: &[u8]) -> (usize, usize) {
    let crlf_idx = match input.iter().position(|&c| c == b'\r') {
        Some(idx) => idx,
        None => panic!("Failed to find crlf in input"),
    };
    let len_str = match std::str::from_utf8(&input[1..crlf_idx]){
        Ok(s) => s,
        Err(_) => panic!("Failed to parse length string into utf8 string"),
    };
    let len: usize = match len_str.parse() {
        Ok(n) => n,
        Err(_) => panic!("Failed to parse length string into integer"),
    };
    (len, crlf_idx + 2)
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

    #[test]
    fn test_deserialize_simple_string() {
        let input = b"+OK\r\n";
        let expected = RespType::SimpleString("OK".to_string());
        assert_eq!(deserialize(input), (expected, input.len()));
    }

    #[test]
    fn test_deserialize_null_array(){
        let input = NULL_ARRAY;
        let expected = RespType::Array(None);
        assert_eq!(deserialize(input), (expected, input.len()));
    }

    #[test]
    fn test_deserialize_null_bulk_string(){
        let input = NULL_BULK_STRING;
        let expected = RespType::BulkString(None);
        assert_eq!(deserialize(input), (expected, input.len()));
    }

    #[test]
    fn test_deserialize_array_simple() {
        let input = b"*2\r\n+foo\r\n-bar\r\n";
        let expected = RespType::Array(Some(vec![
            RespType::SimpleString("foo".to_string()),
            RespType::Error("bar".to_string()),

        ]));
        assert_eq!(deserialize(input), (expected, input.len()));
    }

    #[test]
    fn test_deserialize_array_of_bulk_string() {
        let input = b"*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n";
        let expected = RespType::Array(Some(vec![
            RespType::BulkString(Some("foo".to_string().into_bytes())),
            RespType::BulkString(Some("bar".to_string().into_bytes())),
        ]));
        assert_eq!(deserialize(input), (expected, input.len()));
    }

    #[test]
    fn test_deserialize_nested_array(){
        let input = b"*2\r\n*2\r\n+foo\r\n-bar\r\n*2\r\n+foo\r\n-bar\r\n";
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
        assert_eq!(deserialize(input), (expected, input.len()));
    }

    #[test]
    fn test_deserialize_bulk_string_with_newline(){
        let input = b"$8\r\nfoo\r\nbar\r\n";
        let expected = RespType::BulkString(Some("foo\r\nbar".to_string().into_bytes()));
        assert_eq!(deserialize(input), (expected, input.len()));
    }

    #[test]
    fn test_deserialize_mixed_array() {
        let input = b"*3\r\n+foo\r\n$3\r\nbar\r\n:42\r\n";
        let expected = RespType::Array(Some(vec![
            RespType::SimpleString("foo".to_string()),
            RespType::BulkString(Some("bar".to_string().into_bytes())),
            RespType::Integer(42),
        ]));
        assert_eq!(deserialize(input), (expected, input.len()));
    }
    
    #[test]
    fn test_deserialize_bulk_string_wrong_length(){
        let input = b"$3\r\nfooooooo\r\n";
    }
}
