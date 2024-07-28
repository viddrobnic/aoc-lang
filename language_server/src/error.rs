use core::fmt;

use thiserror::Error;

use crate::RequestId;

#[derive(Debug)]
pub enum ErrorKind {
    ExtractError(serde_json::Error),
}

#[derive(Debug, Error)]
pub struct Error {
    pub request_id: RequestId,
    pub kind: ErrorKind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error for request: {:?}: {}", self.request_id, self.kind)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::ExtractError(err) => write!(f, "Failed to extract params: {}", err),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
