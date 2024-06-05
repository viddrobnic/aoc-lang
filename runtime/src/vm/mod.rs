use std::{collections::HashMap, rc::Rc};

use crate::{
    bytecode::{Bytecode, Instruction},
    error::{Error, ErrorKind},
    object::{Array, Closure, DataType, Dictionary, HashKey, Object},
};

use self::frame::Frame;
use self::gc::GarbageCollector;

mod frame;
pub(crate) mod gc;

#[cfg(test)]
mod test;

const STACK_SIZE: usize = 4096;
const GLOBALS_SIZE: usize = 512;

#[derive(Debug)]
pub struct VirtualMachine {
    gc: GarbageCollector,

    globals: Vec<Object>,

    frames: Vec<Frame>,
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
            frames: vec![],
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

    fn current_frame(&self) -> &Frame {
        self.frames
            .last()
            .expect("There should be at least one frame on vm stack")
    }

    fn current_frame_mut(&mut self) -> &mut Frame {
        self.frames
            .last_mut()
            .expect("There should be at least one frame on vm stack")
    }

    /// Runs the program.
    pub fn run(&mut self, bytecode: &Bytecode) -> Result<(), Error> {
        let main_closure = Closure {
            function_index: bytecode.main_function,
            free_variables: Rc::new(vec![]),
        };
        let main_frame = Frame {
            closure: main_closure,
            ip: 0,
            base_pointer: 0,
        };
        self.frames.push(main_frame);

        loop {
            let ip = self.current_frame().ip;
            let function = &bytecode.functions[self.current_frame().closure.function_index];

            if ip >= function.instructions.len() {
                break;
            }

            let new_ip = self
                .execute_instruction(ip, &function.instructions, &bytecode.constants)
                .map_err(|kind| Error {
                    kind,
                    range: function.ranges[ip],
                })?;
            self.current_frame_mut().ip = new_ip;

            if self.gc.should_free() {
                self.gc.free(&self.stack[0..self.sp]);
            }
        }

        Ok(())
    }

    fn execute_instruction(
        &mut self,
        ip: usize,
        instructions: &[Instruction],
        constants: &[Object],
    ) -> Result<usize, ErrorKind> {
        match instructions[ip] {
            Instruction::Null => self.push(Object::Null)?,
            Instruction::Constant(idx) => self.push(constants[idx].clone())?,
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
            Instruction::IndexGet => self.index_get()?,
            Instruction::Add => self.execute_add()?,
            Instruction::Subtract => self.execute_infix_number_op(
                |left, right| left - right,
                |left, right| left - right,
                ErrorKind::InvalidSubtractType,
            )?,
            Instruction::Multiply => self.execute_infix_number_op(
                |left, right| left * right,
                |left, right| left * right,
                ErrorKind::InvalidMultiplyType,
            )?,
            Instruction::Divide => self.execute_infix_number_op(
                |left, right| left / right,
                |left, right| left / right,
                ErrorKind::InvalidDivideType,
            )?,
            Instruction::Modulo => self.execute_modulo()?,
            Instruction::And => self.execute_and()?,
            Instruction::Or => self.execute_or()?,
            Instruction::Le => self.execute_le()?,
            Instruction::Leq => self.execute_leq()?,
            Instruction::Eq => self.execute_eq()?,
            Instruction::Neq => self.execute_neq()?,
            Instruction::Return => todo!(),
            Instruction::CreateClosure {
                function_index,
                nr_free_variables,
            } => self.create_closure(function_index, nr_free_variables)?,
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

                // TODO: Handle out of bounds
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

    fn index_get(&mut self) -> Result<(), ErrorKind> {
        let index = self.pop();
        let container = self.pop();
        match container {
            Object::Array(arr) => {
                let Object::Integer(idx) = index else {
                    return Err(ErrorKind::InvalidIndexType(index.into()));
                };

                if idx < 0 {
                    self.push(Object::Null)?;
                    return Ok(());
                }

                let rc = arr.0.value.upgrade().unwrap();
                let slice = rc.borrow();

                match slice.get(idx as usize) {
                    Some(obj) => self.push(obj.clone())?,
                    None => self.push(Object::Null)?,
                }
            }
            Object::Dictionary(dict) => {
                let key: HashKey = index.try_into()?;

                let rc = dict.0.value.upgrade().unwrap();
                let hmap = rc.borrow();
                match hmap.get(&key) {
                    Some(obj) => self.push(obj.clone())?,
                    None => self.push(Object::Null)?,
                }
            }

            _ => return Err(ErrorKind::NotIndexable(container.into())),
        }

        Ok(())
    }

    fn execute_add(&mut self) -> Result<(), ErrorKind> {
        let right_obj = self.pop();
        let left_obj = self.pop();

        match (&left_obj, &right_obj) {
            (Object::Integer(left), Object::Integer(right)) => {
                self.push(Object::Integer(left + right))?;
            }
            (Object::Float(left), Object::Float(right)) => {
                self.push(Object::Float(left + right))?;
            }
            (Object::String(left), Object::String(right)) => {
                let mut res = left.to_string();
                res += right;
                self.push(Object::String(Rc::new(res)))?;
            }
            (Object::Array(left), Object::Array(right)) => {
                let left_rc = left.0.value.upgrade().unwrap();
                let right_rc = right.0.value.upgrade().unwrap();

                let mut res = left_rc.borrow().to_vec();
                res.extend_from_slice(&right_rc.borrow());

                let res_ref = self.gc.allocate(res);
                self.push(Object::Array(Array(res_ref)))?;
            }

            _ => return Err(ErrorKind::InvalidAddType(left_obj.into(), right_obj.into())),
        }
        Ok(())
    }

    fn execute_infix_number_op<I, F, E>(
        &mut self,
        int_fn: I,
        float_fn: F,
        err: E,
    ) -> Result<(), ErrorKind>
    where
        I: Fn(i64, i64) -> i64,
        F: Fn(f64, f64) -> f64,
        E: Fn(DataType, DataType) -> ErrorKind,
    {
        let right_obj = self.pop();
        let left_obj = self.pop();

        match (&left_obj, &right_obj) {
            (Object::Integer(left), Object::Integer(right)) => {
                self.push(Object::Integer(int_fn(*left, *right)))?;
            }
            (Object::Float(left), Object::Float(right)) => {
                self.push(Object::Float(float_fn(*left, *right)))?;
            }

            _ => return Err(err(left_obj.into(), right_obj.into())),
        }

        Ok(())
    }

    fn execute_modulo(&mut self) -> Result<(), ErrorKind> {
        let right_obj = self.pop();
        let left_obj = self.pop();

        match (&left_obj, &right_obj) {
            (Object::Integer(left), Object::Integer(right)) => {
                self.push(Object::Integer(left.rem_euclid(*right)))?;
            }
            _ => {
                return Err(ErrorKind::InvalidModuloType(
                    left_obj.into(),
                    right_obj.into(),
                ))
            }
        }

        Ok(())
    }

    fn execute_and(&mut self) -> Result<(), ErrorKind> {
        let right_obj = self.pop();
        let left_obj = self.pop();

        match (&left_obj, &right_obj) {
            (Object::Integer(left), Object::Integer(right)) => {
                self.push(Object::Integer(left & right))?;
            }
            (Object::Boolean(left), Object::Boolean(right)) => {
                self.push(Object::Boolean(*left && *right))?;
            }

            _ => return Err(ErrorKind::InvalidAndType(left_obj.into(), right_obj.into())),
        }

        Ok(())
    }

    fn execute_or(&mut self) -> Result<(), ErrorKind> {
        let right_obj = self.pop();
        let left_obj = self.pop();

        match (&left_obj, &right_obj) {
            (Object::Integer(left), Object::Integer(right)) => {
                self.push(Object::Integer(left | right))?;
            }
            (Object::Boolean(left), Object::Boolean(right)) => {
                self.push(Object::Boolean(*left || *right))?;
            }

            _ => return Err(ErrorKind::InvalidOrType(left_obj.into(), right_obj.into())),
        }

        Ok(())
    }

    fn execute_le(&mut self) -> Result<(), ErrorKind> {
        let right_obj = self.pop();
        let left_obj = self.pop();

        match (&left_obj, &right_obj) {
            (Object::Integer(left), Object::Integer(right)) => {
                self.push(Object::Boolean(left < right))?;
            }
            (Object::Float(left), Object::Float(right)) => {
                self.push(Object::Boolean(left < right))?;
            }
            (Object::String(left), Object::String(right)) => {
                self.push(Object::Boolean(left < right))?;
            }

            _ => {
                return Err(ErrorKind::InvalidOrderingType(
                    left_obj.into(),
                    right_obj.into(),
                ))
            }
        }

        Ok(())
    }

    fn execute_leq(&mut self) -> Result<(), ErrorKind> {
        let right_obj = self.pop();
        let left_obj = self.pop();

        match (&left_obj, &right_obj) {
            (Object::Integer(left), Object::Integer(right)) => {
                self.push(Object::Boolean(left <= right))?;
            }
            (Object::Float(left), Object::Float(right)) => {
                self.push(Object::Boolean(left <= right))?;
            }
            (Object::String(left), Object::String(right)) => {
                self.push(Object::Boolean(left <= right))?;
            }

            _ => {
                return Err(ErrorKind::InvalidOrderingType(
                    left_obj.into(),
                    right_obj.into(),
                ))
            }
        }

        Ok(())
    }

    fn execute_eq(&mut self) -> Result<(), ErrorKind> {
        let right_obj = self.pop();
        let left_obj = self.pop();

        match (&left_obj, &right_obj) {
            (Object::Integer(left), Object::Integer(right)) => {
                self.push(Object::Boolean(left == right))?;
            }
            (Object::Float(left), Object::Float(right)) => {
                self.push(Object::Boolean(left == right))?;
            }
            (Object::Boolean(left), Object::Boolean(right)) => {
                self.push(Object::Boolean(left == right))?;
            }
            (Object::String(left), Object::String(right)) => {
                self.push(Object::Boolean(left == right))?;
            }

            _ => {
                return Err(ErrorKind::InvalidEqualityType(
                    left_obj.into(),
                    right_obj.into(),
                ))
            }
        }

        Ok(())
    }

    fn execute_neq(&mut self) -> Result<(), ErrorKind> {
        let right_obj = self.pop();
        let left_obj = self.pop();

        match (&left_obj, &right_obj) {
            (Object::Integer(left), Object::Integer(right)) => {
                self.push(Object::Boolean(left != right))?;
            }
            (Object::Float(left), Object::Float(right)) => {
                self.push(Object::Boolean(left != right))?;
            }
            (Object::Boolean(left), Object::Boolean(right)) => {
                self.push(Object::Boolean(left != right))?;
            }
            (Object::String(left), Object::String(right)) => {
                self.push(Object::Boolean(left != right))?;
            }

            _ => {
                return Err(ErrorKind::InvalidEqualityType(
                    left_obj.into(),
                    right_obj.into(),
                ))
            }
        }

        Ok(())
    }

    fn create_closure(
        &mut self,
        function_index: usize,
        _nr_free_variables: usize,
    ) -> Result<(), ErrorKind> {
        // TODO: handle capture of free variables.
        let cl = Object::Closure(Closure {
            function_index,
            free_variables: Rc::new(vec![]),
        });
        self.push(cl)?;

        Ok(())
    }
}
