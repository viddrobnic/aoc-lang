use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Null,
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(Rc<String>),
    Array(Rc<Vec<Object>>),
}
