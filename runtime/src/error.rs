use std::fmt::Display;

use parser::position::Range;
use thiserror::Error;

use crate::object::DataType;

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    StackOverflow,
    NotHashable(DataType),
    InvalidNegateOperand(DataType),
    UndefinedSymbol(String),
    NotUnpackable(DataType),
    UnpackLengthMismatch { expected: usize, got: usize },
    UnpackTooLarge { max: usize, got: usize },
    NotIndexable(DataType),
    InvalidIndexType(DataType),
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
            ErrorKind::UndefinedSymbol(ident) => write!(f, "Symbol {ident} is not defined"),
            ErrorKind::NotUnpackable(dt) => write!(f, "Data type {dt} can't be unpacked"),
            ErrorKind::UnpackLengthMismatch { expected, got } => write!(
                f,
                "Invalid number of elements to unpack. Expected: {expected}, got: {got}"
            ),
            ErrorKind::UnpackTooLarge { max, got } => write!(
                f,
                "Too many elements to unpack. Max allowed: {max}, got: {got}"
            ),
            ErrorKind::NotIndexable(dt) => write!(f, "Data type {dt} can't be indexed"),
            ErrorKind::InvalidIndexType(dt) => write!(f, "Invalid index type: {dt}"),
        }
    }
}
