use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(Rc<String>),
}