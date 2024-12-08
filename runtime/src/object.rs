use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::builtin::Builtin;
use crate::error::ErrorKind;
use crate::vm::gc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Char(u8),
    String(Rc<String>),
    Array(Array),
    Dictionary(Dictionary),
    Closure(Closure),
    Builtin(Builtin),
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        !matches!(self, Object::Null | Object::Boolean(false))
    }
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

#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    // Index of the function in the Bytecode.functions
    pub(crate) function_index: usize,

    // Captured variables. They have to be Rc, since a
    // closure can be on stack multiple times. But it doesn't have
    // to be managed by GC, since it's immutable and we can't create
    // a cycle closures alone.
    pub(crate) free_variables: Rc<Vec<Object>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum HashKey {
    Integer(i64),
    Boolean(bool),
    String(Rc<String>),
    Char(u8),
}

impl TryFrom<Object> for HashKey {
    type Error = ErrorKind;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        match value {
            Object::String(str) => Ok(Self::String(str)),
            Object::Integer(i) => Ok(Self::Integer(i)),
            Object::Boolean(b) => Ok(Self::Boolean(b)),
            Object::Char(c) => Ok(Self::Char(c)),

            _ => Err(ErrorKind::NotHashable(value.into())),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DataType {
    Null,
    Integer,
    Float,
    Char,
    Boolean,
    String,
    Array,
    HashMap,
    Closure,
    Builtin,
}

impl From<&Object> for DataType {
    fn from(value: &Object) -> Self {
        match value {
            Object::Null => Self::Null,
            Object::Integer(_) => Self::Integer,
            Object::Float(_) => Self::Float,
            Object::Char(_) => Self::Char,
            Object::Boolean(_) => Self::Boolean,
            Object::String(_) => Self::String,
            Object::Array(_) => Self::Array,
            Object::Dictionary(_) => Self::HashMap,
            Object::Closure(_) => Self::Closure,
            Object::Builtin(_) => Self::Builtin,
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
            DataType::Char => write!(f, "CHAR"),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::String => write!(f, "STRING"),
            DataType::Array => write!(f, "ARRAY"),
            DataType::HashMap => write!(f, "HASH_MAP"),
            DataType::Closure => write!(f, "CLOSURE"),
            DataType::Builtin => write!(f, "BUILTIN"),
        }
    }
}
