use crate::object::Object;

use parser::position::Range;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    Pop,
    Constant(usize),
    Array(usize),
    HashMap(usize),

    Minus,
    Bang,

    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    And,
    Or,
    Le,
    Leq,
    Eq,
    Neq,

    Jump(usize),
    JumpNotTruthy(usize),

    // Puts all array values on stack, where
    // array should be given size long.
    UnpackArray(usize),

    IndexSet,

    StoreGlobal(usize),
    LoadGlobal(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bytecode {
    pub constants: Vec<Object>,

    pub instructions: Vec<Instruction>,
    pub ranges: Vec<Range>,
}
