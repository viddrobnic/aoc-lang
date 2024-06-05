use crate::object::Object;

use parser::position::Range;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Instruction {
    Pop,
    Null,
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

    Return,
    CreateClosure {
        function_index: usize,
        nr_free_variables: usize,
    },

    // Puts all array values on stack, where
    // array should be given size long.
    UnpackArray(usize),

    IndexSet,
    IndexGet,

    StoreGlobal(usize),
    LoadGlobal(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub instructions: Vec<Instruction>,
    pub ranges: Vec<Range>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bytecode {
    pub constants: Vec<Object>,
    pub functions: Vec<Function>,

    pub instructions: Vec<Instruction>,
    pub ranges: Vec<Range>,
}
