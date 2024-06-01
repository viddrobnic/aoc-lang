use std::{collections::HashMap, rc::Rc};

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(Rc<String>),
    Array(Rc<Vec<Object>>),
    HashMap(Rc<HashMap<HashKey, Object>>),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum HashKey {
    Integer(i64),
    Boolean(bool),
    String(Rc<String>),
}

impl TryFrom<Object> for HashKey {
    type Error = ();

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        match value {
            Object::String(str) => Ok(Self::String(str)),
            Object::Integer(i) => Ok(Self::Integer(i)),
            Object::Boolean(b) => Ok(Self::Boolean(b)),
            _ => panic!("unhashable key"),
        }
    }
}
