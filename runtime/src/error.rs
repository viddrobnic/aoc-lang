use std::fmt::Display;

use parser::position::Range;
use thiserror::Error;

use crate::{builtin::Builtin, object::DataType};

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    StackOverflow,
    NotHashable(DataType),
    InvalidNegateOperand(DataType),
    UndefinedSymbol(String),
    NotUnpackable(DataType),
    UnpackLengthMismatch {
        expected: usize,
        got: usize,
    },
    UnpackTooLarge {
        max: usize,
        got: usize,
    },
    NotIndexable(DataType),
    ControlFlowOutsideOfLoop,
    ReturnOutsideOfFunction,
    InvalidImportPath(String),
    ImportParserError {
        path: String,
        error: parser::error::Error,
    },
    ImportCompilerError {
        path: String,
        error: Box<Error>,
    },

    InvalidIndexType(DataType),
    InvalidAddType(DataType, DataType),
    InvalidSubtractType(DataType, DataType),
    InvalidMultiplyType(DataType, DataType),
    InvalidDivideType(DataType, DataType),
    InvalidModuloType(DataType, DataType),
    InvalidAndType(DataType, DataType),
    InvalidOrType(DataType, DataType),
    InvalidOrderingType(DataType, DataType),
    InvalidEqualityType(DataType, DataType),
    InvalidFunctionCalee(DataType),
    InvalidNrOfArgs {
        expected: usize,
        got: usize,
    },
    IndexOutOfBounds,

    InvalidBuiltinArg {
        builtin: Builtin,
        data_type: DataType,
    },
    InputError,
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
            ErrorKind::InvalidImportPath(path)=>write!(f, "File {path} could not be imported"),
            ErrorKind::ImportParserError{path, error}=>write!(f, "Parser error during import {path}: {error}"),
            ErrorKind::ImportCompilerError { path, error }=>write!(f, "Compiler error during import {path}: {error}"),

            ErrorKind::InvalidIndexType(dt) => write!(f, "Invalid index type: {dt}"),

            ErrorKind::InvalidAddType(left, right) => write!(
                f,
                "Can't perform {left} + {right}. Can add integers, floats, strings and arrays."
            ),
            ErrorKind::InvalidSubtractType(left, right) => write!(
                f,
                "Can't perform {left} - {right}. Can subtract integers and floats."
            ),
            ErrorKind::InvalidMultiplyType(left, right) => write!(
                f,
                "Can't perform {left} * {right}. Can multiply integers and floats."
            ),
            ErrorKind::InvalidDivideType(left, right) => write!(
                f,
                "Can't perform {left} / {right}. Can divide integers and floats."
            ),
            ErrorKind::InvalidModuloType(left, right) => write!(
                f,
                "Can't perform {left} % {right}. Can calculate moduleo of integers."
            ),
            ErrorKind::InvalidAndType(left, right) => write!(
                f,
                "Can't perform {left} & {right}. Can perform and on integers and booleans"
            ),
            ErrorKind::InvalidOrType(left, right) => write!(
                f,
                "Can't perform {left} | {right}. Can perform or on integers and booleans"
            ),
            ErrorKind::InvalidOrderingType(left, right) => write!(
                f,
                "Can't compare order of {left} and {right}. Can compare order of integers, floats and strings."
            ),
            ErrorKind::InvalidEqualityType(left, right) => write!(
                f,
                "Can't compare equality of {left} and {right}. Can compare equality of integers, floats, booleans and strings."
            ),
            ErrorKind::ControlFlowOutsideOfLoop => write!(
                f,
                "Break and continue can be used only inside of for and while loops."
            ),
            ErrorKind::ReturnOutsideOfFunction => write!(f, "Return can't be used outside of a function."),
            ErrorKind::InvalidFunctionCalee(dt) => write!(f,"Can only call functions, not {dt}"),
            ErrorKind::InvalidNrOfArgs { expected, got } => write!(f, "Invalid number of arguments, expected: {expected}, got: {got}"),
            ErrorKind::IndexOutOfBounds => write!(f, "Index you are assigning to is out of bounds"),

            ErrorKind::InvalidBuiltinArg { builtin, data_type } => write!(f, "Can't call {builtin} on {data_type}."),
            ErrorKind::InputError => write!(f, "Could not read from stdin"),
        }
    }
}
