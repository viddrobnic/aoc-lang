use std::collections::HashMap;

use crate::{
    bytecode::{Bytecode, Instruction},
    error::{Error, ErrorKind},
    object::{Array, Dictionary, HashKey, Object},
};

use self::gc::GarbageCollector;

pub(crate) mod gc;

#[cfg(test)]
mod test;

const STACK_SIZE: usize = 4096;
const GLOBALS_SIZE: usize = 512;

#[derive(Debug)]
pub struct VirtualMachine {
    gc: GarbageCollector,

    globals: Vec<Object>,

    stack: Vec<Object>,
    // StackPointer which points to the next value.
    // Top of the stack is stack[sp-1]
    sp: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            gc: GarbageCollector::new(),
            globals: vec![Object::Null; GLOBALS_SIZE],
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
        let mut ip = 0;
        while ip < bytecode.instructions.len() {
            ip = self
                .execute_instruction(ip, bytecode)
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

    fn execute_instruction(&mut self, ip: usize, bytecode: &Bytecode) -> Result<usize, ErrorKind> {
        match bytecode.instructions[ip] {
            Instruction::Constant(idx) => self.push(bytecode.constants[idx].clone())?,
            Instruction::Pop => {
                self.pop();
            }
            Instruction::Array(len) => self.execute_array(len)?,
            Instruction::HashMap(len) => self.execute_hash_map(len)?,
            Instruction::Minus => self.execute_minus()?,
            Instruction::Bang => self.execute_bang()?,
            Instruction::Jump(index) => return Ok(index),
            Instruction::JumpNotTruthy(index) => {
                let obj = self.pop();
                if !obj.is_truthy() {
                    return Ok(index);
                }
            }
            Instruction::UnpackArray(size) => self.unpack_array(size)?,
            Instruction::StoreGlobal(index) => {
                let obj = self.pop();
                self.globals[index] = obj;
            }
            Instruction::LoadGlobal(index) => {
                let obj = self.globals[index].clone();
                self.push(obj)?;
            }
            Instruction::IndexSet => self.index_set()?,
        }

        Ok(ip + 1)
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

    fn unpack_array(&mut self, size: usize) -> Result<(), ErrorKind> {
        let obj = self.pop();
        let Object::Array(arr) = obj else {
            return Err(ErrorKind::NotUnpackable(obj.into()));
        };

        let rc = arr.0.value.upgrade().unwrap();
        let values = rc.borrow();
        if values.len() != size {
            return Err(ErrorKind::UnpackLengthMismatch {
                expected: size,
                got: values.len(),
            });
        }

        if values.len() > 256 {
            return Err(ErrorKind::UnpackTooLarge {
                max: 256,
                got: values.len(),
            });
        }

        for obj in values.iter().rev() {
            self.push(obj.clone())?;
        }

        Ok(())
    }

    fn index_set(&mut self) -> Result<(), ErrorKind> {
        let index = self.pop();
        let container = self.pop();
        let value = self.pop();

        match container {
            Object::Array(arr) => {
                let Object::Integer(idx) = index else {
                    return Err(ErrorKind::InvalidIndexType(index.into()));
                };

                let rc = arr.0.value.upgrade().unwrap();
                rc.borrow_mut()[idx as usize] = value;
            }
            Object::Dictionary(dict) => {
                let key: HashKey = index.try_into()?;

                let rc = dict.0.value.upgrade().unwrap();
                rc.borrow_mut().insert(key, value);
            }

            _ => return Err(ErrorKind::NotIndexable(container.into())),
        }

        Ok(())
    }
}
