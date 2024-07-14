use regex::Regex;

pub const CRLF: &str = "\r\n";
pub const NULL_BULK_STRING: &str = "$-1\r\n";
pub const NULL_ARRAY: &str = "*-1\r\n";
use once_cell::sync::Lazy;

// pub static RESP_REGEX: Regex = Regex::new(r"(\+|-|:|\$|\*).*(\r\n)").unwrap();
pub static CRLF_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\r\n").unwrap());
pub static RARRAY_HEAD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\*([0-9]+)(\r\n)").unwrap());

pub trait RespValue {
    fn serialize(&self) -> String;
}

#[derive(Debug, PartialEq)]
pub struct SimpleString(pub String);

impl RespValue for SimpleString {
    fn serialize(&self) -> String {
        format!("+{}\r\n", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct RError(pub String);

impl RespValue for RError {
    fn serialize(&self) -> String {
        format!("-{}\r\n", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct RInteger(pub i64);

impl  RespValue for RInteger {
    fn serialize(&self) -> String {
        format!(":{}\r\n", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub struct BulkString(pub Option<Vec<u8>>);

impl RespValue for BulkString {
    fn serialize(&self) -> String {
        match &self.0 {
            None => "$-1\r\n".to_string(),
            Some(b) => format!("${}\r\n{}\r\n", b.len(), String::from_utf8_lossy(b))
        }
    }
}

// An RArray can itself be null and its elements can be null too.
// The elements should be unparsed RESP strings 
#[derive(Debug, PartialEq)]
pub struct RArray(pub Option<Vec<String>>);

impl RespValue for RArray {
    fn serialize(&self) -> String {
        match &self.0 {
            None => "*-1\r\n".to_string(),
            Some(v) => {
                format!("*{}\r\n{}", v.len(), v.join(""))
            }
        }
    }
}

impl RArray {
    pub fn next(&self) -> Option<&str> {
        match &self.0 {
            None => None,
            Some(v) => {
                if v.len() == 0 {
                    None
                } else {
                    Some(&v[0])
                }
            }
        }
    }
}
