use std::fmt::Display;

use parser::position::Range;
use thiserror::Error;

use crate::object::DataType;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    StackOverflow,
    NotHashable(DataType),
    InvalidNegateOperand(DataType),
}

#[derive(Debug, Error, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub range: Range,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::StackOverflow => write!(f, "Stack overflow"),
            ErrorKind::NotHashable(data_type) => write!(f, "Data type {data_type} can't be hashed"),
            ErrorKind::InvalidNegateOperand(dt) => write!(f, "Can not negate {dt}"),
        }
    }
}
