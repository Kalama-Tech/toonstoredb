//! Error types for toonstoredb

use std::fmt;
use std::io;

/// Result type alias for toonstoredb operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for database operations
#[derive(Debug)]
pub enum Error {
    /// I/O error
    Io(io::Error),
    
    /// Parse error
    Parse(String),
    
    /// Value too large (max 1 MB)
    ValueTooLarge(usize),
    
    /// Database full (max 1 GB)
    DatabaseFull(u64),
    
    /// Key not found
    NotFound,
    
    /// Database is closed
    Closed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Parse(msg) => write!(f, "Parse error: {}", msg),
            Error::ValueTooLarge(size) => write!(f, "Value too large: {} bytes (max 1 MB)", size),
            Error::DatabaseFull(size) => write!(f, "Database full: {} bytes (max 1 GB)", size),
            Error::NotFound => write!(f, "Key not found"),
            Error::Closed => write!(f, "Database is closed"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<nom::Err<nom::error::Error<&[u8]>>> for Error {
    fn from(err: nom::Err<nom::error::Error<&[u8]>>) -> Self {
        Error::Parse(format!("{:?}", err))
    }
}
