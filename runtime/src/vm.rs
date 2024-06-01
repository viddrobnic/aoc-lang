use std::{collections::HashMap, rc::Rc};

use crate::{
    bytecode::{Bytecode, Instruction},
    object::{HashKey, Object},
};

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
        if self.sp == 0 {
            panic!("No more elements");
        }

        self.sp -= 1;
        self.stack[self.sp].clone()
    }

    /// Runs the program and returns the first element on the stack.
    ///
    /// This is primarily used to test if the vm works correctly.
    /// The first element on the stack should be the last popped element
    /// if the compiler and vm both work correctly.
    pub fn run(mut self, bytecode: &Bytecode) -> Object {
        for inst in &bytecode.instructions {
            match inst {
                Instruction::Constant(idx) => self.push(bytecode.constants[*idx].clone()),
                Instruction::Pop => {
                    self.pop();
                }
                Instruction::Array(len) => self.execute_array(*len),
                Instruction::HashMap(len) => self.execute_hash_map(*len),
            }
        }

        self.stack[0].clone()
    }

    fn execute_array(&mut self, len: usize) {
        let start = self.sp - len;

        let arr = self.stack[start..self.sp].to_vec();
        self.sp -= len;

        self.push(Object::Array(Rc::new(arr)));
    }

    fn execute_hash_map(&mut self, len: usize) {
        let start = self.sp - len;

        let hash_map: Result<HashMap<_, _>, _> = self.stack[start..self.sp]
            .chunks(2)
            .map(|chunk| -> Result<(HashKey, Object), ()> {
                let key = &chunk[0];
                let value = &chunk[1];

                let key: HashKey = key.clone().try_into()?;

                Ok((key, value.clone()))
            })
            .collect();

        let hash_map = hash_map.unwrap();

        self.sp -= len;
        self.push(Object::HashMap(Rc::new(hash_map)));
    }
}
