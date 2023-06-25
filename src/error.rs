use std::{error::Error, fmt};

#[derive(Debug)]
pub enum BackendError {
    NotFound,
    BadRequest,
    AlreadyExists,
    Unauthorized,
    BadGateway,
    InternalServerError(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BackendError::NotFound => f.write_str("NotFound"),
            BackendError::BadRequest => f.write_str("BadRequest"),
            BackendError::AlreadyExists => f.write_str("AlreadyExists"),
            BackendError::Unauthorized => f.write_str("Unauthorized"),
            BackendError::BadGateway => f.write_str("BadGateway"),
            BackendError::InternalServerError(err) => f.write_str(err.as_str()),
        }
    }
}

impl Error for BackendError {
    fn description(&self) -> &str {
        match self {
            BackendError::NotFound => "Resource not found",
            BackendError::BadRequest => "Bad request",
            BackendError::AlreadyExists => "Resource already exists",
            BackendError::Unauthorized => "Unauthorized",
            BackendError::BadGateway => "BadGateway",
            BackendError::InternalServerError(err) => err.as_str(),
        }
    }
}
