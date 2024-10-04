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
}

impl Error for DataStoreError {}

impl Display for DataStoreError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DataStoreError::LockError => write!(f, "Failed to acquire lock"),
            DataStoreError::KeyNotFound => write!(f, "Key not found"),
        }
    }
    
}