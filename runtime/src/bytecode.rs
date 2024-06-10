use crate::{builtin::Builtin, object::Object};

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
    CreateClosure(CreateClosure),
    FnCall(usize),

    // Puts all array values on stack, where
    // array should be given size long.
    UnpackArray(usize),

    IndexSet,
    IndexGet,

    StoreGlobal(usize),
    LoadGlobal(usize),
    StoreLocal(usize),
    LoadLocal(usize),
    LoadFree(usize),
    CurrentClosure,
    Builtin(Builtin),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CreateClosure {
    pub function_index: usize,
    pub nr_free_variables: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub instructions: Vec<Instruction>,
    pub ranges: Vec<Range>,

    pub nr_local_variables: usize,
    pub nr_arguments: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bytecode {
    pub constants: Vec<Object>,
    pub functions: Vec<Function>,

    // Index of the main function
    pub main_function: usize,
}
