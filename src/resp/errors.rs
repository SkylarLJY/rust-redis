use core::fmt;
use std::{error::Error, fmt::{Display, Formatter}};

#[derive(Debug, PartialEq)]
pub enum DeserializeError {
    InvalidInput(String),
    LengthMismatch(String),
    EmptyInput,
}

impl Error for DeserializeError {}

impl Display for DeserializeError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DeserializeError::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            DeserializeError::LengthMismatch(s) => write!(f, "Length mismatch: {}", s),
            DeserializeError::EmptyInput => write!(f, "Empty input"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DataStoreError {
    LockError,
    KeyNotFound,
    SerializeError,
    FileIOError,
    DataLoadError,
}

impl Error for DataStoreError {}

impl Display for DataStoreError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DataStoreError::LockError => write!(f, "Failed to acquire lock"),
            DataStoreError::KeyNotFound => write!(f, "Key not found"),
            DataStoreError::SerializeError => write!(f, "Failed to serialize data"),
            DataStoreError::FileIOError => write!(f, "Failed to read/write file"),
            DataStoreError::DataLoadError => write!(f, "Failed to load data"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ServerError {
    AcceptError,
    ReadError,
    RespParseError(String),
    TypeError,
    DataStoreError(DataStoreError),
    UserInputError(UserInputError),
    ConfigError(String),
}

impl Error for ServerError {}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ServerError::AcceptError => write!(f, "Failed to accept connection"),
            ServerError::ReadError => write!(f, "Failed to read from stream"),
            ServerError::RespParseError(s) => write!(f, "Failed to parse RESP {}", s),
            ServerError::TypeError => write!(f, "Type error"),
            ServerError::DataStoreError(e) => write!(f, "Data store error: {}", e),
            ServerError::UserInputError(e) => write!(f, "User input error: {}", e),
            ServerError::ConfigError(e) => write!(f, "Config error: {}", e),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum UserInputError {
    InvalidInput(String),
    DataStoreError(DataStoreError),
    UnknownCommand(String),
}

impl Error for UserInputError {}
    
impl Display for UserInputError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            UserInputError::InvalidInput(s) => write!(f, "Invalid input: {}", s),
            UserInputError::DataStoreError(e) => write!(f, "Data store error: {}", e),
            UserInputError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
        }
    }
}

