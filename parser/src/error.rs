use std::fmt::Display;

use thiserror::Error;

use crate::position::Position;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    InvalidNumber(String),
    UnexpectedEof,
    InvalidEscapeChar(char),
    InvalidChar(char),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub position: Position,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::InvalidNumber(number) => write!(f, "invalid number: {number}"),
            ErrorKind::UnexpectedEof => write!(f, "unexpected end of file"),
            ErrorKind::InvalidEscapeChar(ch) => write!(f, "invalid escape char '{ch}'"),
            ErrorKind::InvalidChar(ch) => write!(f, "invalid char '{ch}'"),
        }
    }
}
