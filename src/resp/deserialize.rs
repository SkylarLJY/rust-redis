use super::resp_value::RArray;
use super::resp_value::BulkString;
use super::resp_value::RError;
use super::resp_value::RInteger;
use super::resp_value::RespValue;
use super::resp_value::SimpleString;
use super::resp_value::CRLF;
use super::resp_value::NULL_ARRAY;
use super::resp_value::NULL_BULK_STRING;

// Clients send commands to a Redis Server as a RESP Array of Bulk Strings.
pub fn deserialize_request(input: String) -> RArray {
    parse_array(&input)
}

fn parse_array(input: &str) -> RArray {
    if input == NULL_ARRAY {
        return RArray{size: 0, elements: "".to_string()};
    }
    let parts: Vec<&str> = input.splitn(2, CRLF).collect();
    let size_str  = parts[0].chars().skip(1).collect::<String>();
    let size = size_str.parse().unwrap();
    RArray{size, elements: parts[1].to_string()}
}

impl RArray {
    pub fn next(&self) -> (Box<dyn RespValue>, RArray) {
        let (val, elements) = lazy_parse(&self.elements);
        let size = self.size - 1;
        (val, RArray{size, elements})
    }
}

// parse array elements on request
// TODO: get value from Box
fn lazy_parse(input: &str) -> (Box<dyn RespValue>, String){
    let leading_char = input.chars().nth(0).unwrap();
    match leading_char{
        '+' => {
            let res: Vec<&str> = input.splitn(2, CRLF).collect();
            (Box::new(SimpleString(res[0][1..].to_string())), res[1].to_string())
        }
        '-' => {
            let res: Vec<&str> = input.splitn(2, CRLF).collect();
            (Box::new(RError(res[0][1..].to_string())), res[1].to_string())
        }
        ':' => {
            let res: Vec<&str> = input.splitn(2, CRLF).collect();
            let val: i64 = res[0][1..].parse().unwrap();
            (Box::new(RInteger(val)), res[1].to_string())
        }
        '$' => {
            if input == NULL_BULK_STRING {
                return (Box::new(BulkString(None)), "".to_string());
            }
            let res: Vec<&str> = input.splitn(3, CRLF).collect();
            let len_str: String = res[0].chars().skip(1).collect();
            let len: isize = len_str.parse().unwrap();
            let val = res[1].to_string();
            (Box::new(BulkString(Some(val.into_bytes()))), res[2].to_string())
        
        }
        '*' => todo!("lazy parse array"),
        _ => panic!("Invalid RESP type")
    
    }
}

fn parse_simple_string(input: &str) -> SimpleString {
    let res: Vec<&str> = input.splitn(2, CRLF).collect();
    SimpleString(res[0][1..].to_string())
}

fn parse_error(input: &str) -> RError {
    let res: Vec<&str> = input.split(CRLF).collect();
    RError(res[0][1..].to_string())
}

fn parse_integer(input: &str) -> RInteger {
    let res: Vec<&str> = input.split(CRLF).collect();
    let val: i64 = res[0][1..].parse().unwrap();
    RInteger(val)
}


fn parse_bulk_string(input: &str) -> BulkString {
    if input == "$-1\r\n" {
        return BulkString(None);
    }
    let res: Vec<&str> = input.split(CRLF).collect();
    let len_str: String = res[0].chars().skip(1).collect();
    let len: isize = len_str.parse().unwrap();

    let val = res[1].to_string();
    BulkString(Some(val.into_bytes()))
}

#[cfg(test)]
mod test {
    use crate::resp::resp_value::{BulkString, RArray, RError, RInteger, SimpleString};

    #[test]
    fn test_parse_simple_string() {
        let input = "+OK\r\n".to_string();
        let output = super::parse_simple_string(&input);
        assert_eq!(output, SimpleString("OK".to_string()));
    }

    #[test]
    fn test_parse_error() {
        let input = "-Error message\r\n".to_string();
        let output = super::parse_error(&input);

        assert!(output.eq(&RError("Error message".to_string())));
    }

    #[test]
    fn test_parse_integer() {
        let input = ":1000\r\n".to_string();
        let output = super::parse_integer(&input);
        assert_eq!(output, RInteger(1000));
    }

    #[test]
    fn test_parse_bulk_string() {
        let input = "$6\r\nfoobar\r\n".to_string();
        let output = super::parse_bulk_string(&input);
        assert_eq!(output, BulkString(Some("foobar".as_bytes().to_vec())));
    }

    #[test]
    fn test_parse_array() {
        let input = "*2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n".to_string();
        let output = super::parse_array(&input);
        assert_eq!(
            output,
            RArray{size: 2, elements: "$3\r\nfoo\r\n$3\r\nbar\r\n".to_string()}
        );
    }

    #[test]
    fn test_parse_null_bulkstr() {
        let input = "$-1\r\n".to_string();
        let output = super::parse_bulk_string(&input);
        assert_eq!(output, BulkString(None));
    }

    #[test]
    fn test_parse_null_array() {
        let input = "*-1\r\n".to_string();
        let output = super::parse_array(&input);
        assert_eq!(output, RArray{size: 0, elements: "".to_string()});
    }

    #[test]
    fn test_lazy_parse() {
        let input = "$3\r\nfoo\r\n$3\r\nbar\r\n".to_string();
        let (head, tail) = super::lazy_parse(&input);
        assert_eq!(tail, "$3\r\nbar\r\n".to_string());
    }
}
