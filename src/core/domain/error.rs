use sqlx::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum DomainError {
    NotFoundError(String),
    DatabaseError(String),
    DuplicateError(String),
    InvalidArgument(String),
    ValidationError(String),
}

impl Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // self refers to the instance of the type we are implementing the method for
        match *self {
            // ref creates a reference instead of moving the value
            // ref message creates an immutable reference to the String inside the DomainError,
            // allowing you to access string without taking ownership
            DomainError::NotFoundError(ref msg) => write!(f, "{}", msg),
            DomainError::DatabaseError(ref msg) => write!(f, "{}", msg),
            DomainError::DuplicateError(ref msg) => write!(f, "{}", msg),
            DomainError::InvalidArgument(ref msg) => write!(f, "{}", msg),
            DomainError::ValidationError(ref msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    NotFound,
    UniqueViolation,
    ForeignKeyViolation,
    RecordExists(String),
    // ... other specific database errors
    Configuration(String), // To capture configuration errors
    Tls(String),           // To capture TLS errors
    Protocol(String),      // To capture protocol errors
    Encode(String),        // To capture encoding errors
    Decode(String),        // To capture decoding errors
    PoolTimedOut,
    PoolClosed,
    WorkerCrashed,
    InvalidArgument(String),
    Unknown(String), // Catch-all for other errors with the error message
}

impl From<Error> for DatabaseError {
    fn from(e: Error) -> Self {
        match e {
            Error::RowNotFound => DatabaseError::NotFound,
            Error::Database(e) => {
                if let Some(code) = e.code() {
                    match code.as_ref() {
                        "23505" => DatabaseError::UniqueViolation,
                        "23503" => DatabaseError::ForeignKeyViolation,
                        // ... other specific database error code mappings
                        _ => DatabaseError::Unknown(e.to_string()), // Capture the error message
                    }
                } else {
                    DatabaseError::Unknown(e.to_string()) // Capture the error message
                }
            }
            Error::Configuration(e) => DatabaseError::Configuration(e.to_string()),
            Error::Tls(e) => DatabaseError::Tls(e.to_string()),
            Error::Protocol(e) => DatabaseError::Protocol(e),
            Error::Encode(e) => DatabaseError::Encode(e.to_string()),
            Error::Decode(e) => DatabaseError::Decode(e.to_string()),
            Error::PoolTimedOut => DatabaseError::PoolTimedOut,
            Error::PoolClosed => DatabaseError::PoolClosed,
            Error::WorkerCrashed => DatabaseError::WorkerCrashed,
            // ... other SqlxError variants you want to handle
            _ => DatabaseError::Unknown(e.to_string()), // Catch-all for other errors
        }
    }
}
