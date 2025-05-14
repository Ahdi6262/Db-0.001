use thiserror::Error;
use std::fmt::{self, Display};

/// Custom error type for the unified database API
#[derive(Error, Debug)]
pub enum Error {
    #[error("Database connection error: {0}")]
    DatabaseConnectionError(String),
    
    #[error("Database query error: {0}")]
    DatabaseQueryError(String),
    
    #[error("Database transaction error: {0}")]
    DatabaseTransactionError(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Invalid query: {0}")]
    InvalidQueryError(String),
    
    #[error("Data conversion error: {0}")]
    DataConversionError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Unsupported database type: {0}")]
    UnsupportedDatabaseType(String),
    
    #[error("Database not found: {0}/{1}")]
    DatabaseNotFound(String, String),
    
    #[error("Record not found")]
    RecordNotFound,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),
    
    #[error("UUID error: {0}")]
    UuidError(#[from] uuid::Error),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
