use crate::{bytecode::Bytecode, object::Object};

const STACK_SIZE: usize = 4096;

#[derive(Debug)]
pub struct VirtualMachine {
    stack: Vec<Object>,
    // StackPointer which points to the next value.
    // Top of the stack is stack[sp-1]
    sp: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: vec![Object::Null; STACK_SIZE],
            sp: 0,
        }
    }

    fn push(&mut self, obj: Object) {
        if self.sp >= self.stack.len() {
            panic!("stack overflow");
        }

        self.stack[self.sp] = obj;
        self.sp += 1;
    }

    fn pop(&mut self) -> Object {
        self.sp -= 1;
        self.stack[self.sp].clone()
    }

    pub fn run(&mut self, bytecode: &Bytecode) {
        for inst in &bytecode.instructions {
            match inst {
                crate::bytecode::Instruction::Constant(idx) => {
                    self.push(bytecode.constants[*idx].clone())
                }
                crate::bytecode::Instruction::Pop => {
                    self.pop();
                }
            }
        }
    }
}
