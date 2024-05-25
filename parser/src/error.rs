use std::fmt::Display;

use thiserror::Error;

use crate::{position::Position, token::TokenKind};

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    InvalidNumber(String),
    UnexpectedEof,
    InvalidEscapeChar(char),
    InvalidChar(char),
    InvalidExpression(TokenKind),
    ExpectedEol,
}

#[derive(Debug, Error, PartialEq)]
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
            ErrorKind::InvalidExpression(token) => write!(f, "not a valid expression: {:?}", token),
            ErrorKind::ExpectedEol => write!(f, "expression must end with new line"),
        }
    }
}
