use crate::object::Object;

use parser::position::Range;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    Pop,
    Constant(usize),
    Array(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bytecode {
    pub constants: Vec<Object>,

    pub instructions: Vec<Instruction>,
    pub ranges: Vec<Range>,
}
