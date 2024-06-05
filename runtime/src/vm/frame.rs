use crate::object::Closure;

#[derive(Debug)]
pub struct Frame {
    pub closure: Closure,
    pub ip: usize,
    pub base_pointer: usize,
}
