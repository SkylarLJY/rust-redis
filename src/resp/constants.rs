pub const SIMPLE_STRING_PREFIX: u8 = b'+';
pub const ERROR_PREFIX: u8 = b'-';
pub const INTEGER_PREFIX: u8 = b':';
pub const BULK_STRING_PREFIX: u8 = b'$';
pub const ARRAY_PREFIX: u8 = b'*';
pub const CRLF: &[u8] = b"\r\n";
pub const NULL_BULK_STRING: &[u8] = b"$-1\r\n";
pub const NULL_ARRAY: &[u8] = b"*-1\r\n";