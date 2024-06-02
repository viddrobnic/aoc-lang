use std::collections::HashMap;

use crate::{
    bytecode::{Bytecode, Instruction},
    error::{Error, ErrorKind},
    object::{Array, Dictionary, HashKey, Object},
};

use self::gc::GarbageCollector;

pub(crate) mod gc;

const STACK_SIZE: usize = 4096;

#[derive(Debug)]
pub struct VirtualMachine {
    gc: GarbageCollector,

    stack: Vec<Object>,
    // StackPointer which points to the next value.
    // Top of the stack is stack[sp-1]
    sp: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            gc: GarbageCollector::new(),
            stack: vec![Object::Null; STACK_SIZE],
            sp: 0,
        }
    }

    fn push(&mut self, obj: Object) -> Result<(), ErrorKind> {
        if self.sp >= self.stack.len() {
            return Err(ErrorKind::StackOverflow);
        }

        self.stack[self.sp] = obj;
        self.sp += 1;

        Ok(())
    }

    fn pop(&mut self) -> Object {
        if self.sp == 0 {
            panic!("Trying to pop from empty stack. Something is wrong with compiler or vm...");
        }

        self.sp -= 1;
        self.stack[self.sp].clone()
    }

    /// Runs the program and returns the first element on the stack.
    ///
    /// This is primarily used to test if the vm works correctly.
    /// The first element on the stack should be the last popped element
    /// if the compiler and vm both work correctly.
    pub fn run(&mut self, bytecode: &Bytecode) -> Result<Object, Error> {
        for ip in 0..bytecode.instructions.len() {
            self.execute_instruction(ip, bytecode)
                .map_err(|kind| Error {
                    kind,
                    range: bytecode.ranges[ip],
                })?;

            if self.gc.should_free() {
                self.gc.free(&self.stack[0..self.sp]);
            }
        }

        Ok(self.stack[0].clone())
    }

    fn execute_instruction(&mut self, ip: usize, bytecode: &Bytecode) -> Result<(), ErrorKind> {
        match bytecode.instructions[ip] {
            Instruction::Constant(idx) => self.push(bytecode.constants[idx].clone())?,
            Instruction::Pop => {
                self.pop();
            }
            Instruction::Array(len) => self.execute_array(len)?,
            Instruction::HashMap(len) => self.execute_hash_map(len)?,
            Instruction::Minus => self.execute_minus()?,
            Instruction::Bang => self.execute_bang()?,
        }

        Ok(())
    }

    fn execute_array(&mut self, len: usize) -> Result<(), ErrorKind> {
        let start = self.sp - len;

        let arr = self.stack[start..self.sp].to_vec();
        self.sp -= len;

        let arr_ref = self.gc.allocate(arr);
        self.push(Object::Array(Array(arr_ref)))
    }

    fn execute_hash_map(&mut self, len: usize) -> Result<(), ErrorKind> {
        let start = self.sp - len;

        let hash_map: Result<HashMap<_, _>, _> = self.stack[start..self.sp]
            .chunks(2)
            .map(|chunk| -> Result<(HashKey, Object), ErrorKind> {
                let key = &chunk[0];
                let value = &chunk[1];

                let key: HashKey = key.clone().try_into()?;

                Ok((key, value.clone()))
            })
            .collect();

        self.sp -= len;

        let dict_ref = self.gc.allocate(hash_map?);
        self.push(Object::Dictionary(Dictionary(dict_ref)))
    }

    fn execute_minus(&mut self) -> Result<(), ErrorKind> {
        let value = self.pop();
        match value {
            Object::Integer(int) => self.push(Object::Integer(-int))?,
            Object::Float(float) => self.push(Object::Float(-float))?,

            _ => return Err(ErrorKind::InvalidNegateOperand(value.into())),
        };

        Ok(())
    }

    fn execute_bang(&mut self) -> Result<(), ErrorKind> {
        let value = self.pop();
        match value {
            Object::Boolean(boolean) => self.push(Object::Boolean(!boolean))?,
            Object::Integer(int) => self.push(Object::Integer(!int))?,

            _ => return Err(ErrorKind::InvalidNegateOperand(value.into())),
        };

        Ok(())
    }
}
