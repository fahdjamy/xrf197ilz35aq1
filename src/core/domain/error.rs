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
