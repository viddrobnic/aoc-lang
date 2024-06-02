use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::error::ErrorKind;
use crate::vm::gc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(Rc<String>),
    Array(Array),
    Dictionary(Dictionary),
}

#[derive(Debug, Clone)]
pub struct Array(pub(crate) gc::Ref<Vec<Object>>);

// Implement equality for test purposes.
impl PartialEq for Array {
    fn eq(&self, other: &Self) -> bool {
        // If gc works, we won't be checking non dropped weaks
        // so it's fine to check equality on Option<Rc<...>>
        self.0.value.upgrade() == other.0.value.upgrade()
    }
}

#[derive(Debug, Clone)]
pub struct Dictionary(pub(crate) gc::Ref<HashMap<HashKey, Object>>);

// Similar for array for testing purposes
impl PartialEq for Dictionary {
    fn eq(&self, other: &Self) -> bool {
        self.0.value.upgrade() == other.0.value.upgrade()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum HashKey {
    Integer(i64),
    Boolean(bool),
    String(Rc<String>),
}

impl TryFrom<Object> for HashKey {
    type Error = ErrorKind;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        match value {
            Object::String(str) => Ok(Self::String(str)),
            Object::Integer(i) => Ok(Self::Integer(i)),
            Object::Boolean(b) => Ok(Self::Boolean(b)),

            _ => Err(ErrorKind::NotHashable(value.into())),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataType {
    Null,
    Integer,
    Float,
    Boolean,
    String,
    Array,
    HashMap,
}

impl From<&Object> for DataType {
    fn from(value: &Object) -> Self {
        match value {
            Object::Null => Self::Null,
            Object::Integer(_) => Self::Integer,
            Object::Float(_) => Self::Float,
            Object::Boolean(_) => Self::Boolean,
            Object::String(_) => Self::String,
            Object::Array(_) => Self::Array,
            Object::Dictionary(_) => Self::HashMap,
        }
    }
}

impl From<Object> for DataType {
    fn from(value: Object) -> Self {
        (&value).into()
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Null => write!(f, "NULL"),
            DataType::Integer => write!(f, "INTEGER"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::String => write!(f, "STRING"),
            DataType::Array => write!(f, "ARRAY"),
            DataType::HashMap => write!(f, "HASH_MAP"),
        }
    }
}
