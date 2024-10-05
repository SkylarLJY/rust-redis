pub const SIMPLE_STRING_PREFIX: u8 = b'+';
pub const ERROR_PREFIX: u8 = b'-';
pub const INTEGER_PREFIX: u8 = b':';
pub const BULK_STRING_PREFIX: u8 = b'$';
pub const ARRAY_PREFIX: u8 = b'*';
pub const NULL_BULK_STRING: &[u8] = b"$-1\r\n";
pub const NULL_ARRAY: &[u8] = b"*-1\r\n";

pub const DATA_FILE_PATH: &'static str = ".data.json";
pub const DATA_SAVE_INTERVAL_SECS: u64 = 5;