use std::rc::Rc;

use crate::{
    bytecode::{Bytecode, Instruction},
    error::{Error, ErrorKind},
    object::Object,
};

use parser::{
    ast::{self, PrefixOperatorKind},
    position::Range,
};

use self::symbol_table::{Symbol, SymbolTable};

mod symbol_table;

#[cfg(test)]
mod test;

#[derive(Debug, Default)]
struct Scope {
    instructions: Vec<Instruction>,
    ranges: Vec<Range>,
}

#[derive(Debug)]
pub struct Compiler {
    constants: Vec<Object>,

    symbol_table: SymbolTable,

    scopes: Vec<Scope>,
    scope_index: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            constants: vec![],
            symbol_table: SymbolTable::new(),
            scopes: vec![Scope::default()],
            scope_index: 0,
        }
    }

    fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    fn current_scope(&mut self) -> &mut Scope {
        &mut self.scopes[self.scope_index]
    }

    fn emit(&mut self, instruction: Instruction, range: Range) -> usize {
        let scope = self.current_scope();
        scope.instructions.push(instruction);
        scope.ranges.push(range);

        scope.instructions.len() - 1
    }

    pub fn compile(mut self, program: &ast::Program) -> Result<Bytecode, Error> {
        for node in &program.statements {
            self.compile_node(node)?;

            if node.kind() == ast::NodeKind::Expression {
                self.emit(Instruction::Pop, node.range);
            }
        }

        // If compiler works correctly, we should have one scope.
        let scope = self.scopes.pop().expect("Invalid number of scopes");

        Ok(Bytecode {
            constants: self.constants,
            instructions: scope.instructions,
            ranges: scope.ranges,
        })
    }

    fn compile_node(&mut self, node: &ast::Node) -> Result<(), Error> {
        match &node.value {
            ast::NodeValue::Identifier(ident) => self.compile_ident(ident, node.range)?,
            ast::NodeValue::IntegerLiteral(int) => {
                self.compile_constant(Object::Integer(*int), node.range);
            }
            ast::NodeValue::FloatLiteral(flt) => {
                self.compile_constant(Object::Float(*flt), node.range);
            }
            ast::NodeValue::BoolLiteral(boolean) => {
                self.compile_constant(Object::Boolean(*boolean), node.range);
            }
            ast::NodeValue::StringLiteral(string) => {
                self.compile_constant(Object::String(Rc::new(string.to_string())), node.range);
            }
            ast::NodeValue::ArrayLiteral(arr) => self.compile_array(arr, node.range)?,
            ast::NodeValue::HashLiteral(elements) => self.compile_hash_map(elements, node.range)?,
            ast::NodeValue::PrefixOperator(prefix) => {
                self.compile_prefix_operator(prefix, node.range)?;
            }
            ast::NodeValue::InfixOperator(infix) => {
                self.compile_infix_operator(infix, node.range)?;
            }
            ast::NodeValue::Assign { ident, value } => {
                self.compile_node(value)?;
                self.compile_assign(ident, node.range)?;
            }
            ast::NodeValue::Index { .. } => todo!(),
            ast::NodeValue::If(_) => todo!(),
            ast::NodeValue::While(while_loop) => self.compile_while(while_loop)?,
            ast::NodeValue::For { .. } => todo!(),
            ast::NodeValue::Break => todo!(),
            ast::NodeValue::Continue => todo!(),
            ast::NodeValue::FunctionLiteral { .. } => todo!(),
            ast::NodeValue::FunctionCall { .. } => todo!(),
            ast::NodeValue::Return(_) => todo!(),
            ast::NodeValue::Use(_) => todo!(),
        }

        Ok(())
    }

    fn compile_constant(&mut self, constant: Object, range: Range) {
        let const_idx = self.add_constant(constant);
        self.emit(Instruction::Constant(const_idx), range);
    }

    fn compile_array(&mut self, arr: &[ast::Node], range: Range) -> Result<(), Error> {
        for node in arr {
            self.compile_node(node)?;
        }

        self.emit(Instruction::Array(arr.len()), range);
        Ok(())
    }

    fn compile_hash_map(
        &mut self,
        elements: &[ast::HashLiteralPair],
        range: Range,
    ) -> Result<(), Error> {
        for elt in elements {
            self.compile_node(&elt.key)?;
            self.compile_node(&elt.value)?;
        }

        self.emit(Instruction::HashMap(elements.len() * 2), range);

        Ok(())
    }

    fn compile_prefix_operator(
        &mut self,
        node: &ast::PrefixOperator,
        range: Range,
    ) -> Result<(), Error> {
        self.compile_node(&node.right)?;

        match node.operator {
            PrefixOperatorKind::Not => self.emit(Instruction::Bang, range),
            PrefixOperatorKind::Negative => self.emit(Instruction::Minus, range),
        };

        Ok(())
    }

    fn compile_infix_operator(
        &mut self,
        node: &ast::InfixOperator,
        range: Range,
    ) -> Result<(), Error> {
        let (instruction, reverse) = match node.operator {
            ast::InfixOperatorKind::Add => (Instruction::Add, false),
            ast::InfixOperatorKind::Subtract => (Instruction::Subtract, false),
            ast::InfixOperatorKind::Multiply => (Instruction::Multiply, false),
            ast::InfixOperatorKind::Divide => (Instruction::Divide, false),
            ast::InfixOperatorKind::Modulo => (Instruction::Modulo, false),
            ast::InfixOperatorKind::And => (Instruction::And, false),
            ast::InfixOperatorKind::Or => (Instruction::Or, false),
            ast::InfixOperatorKind::Le => (Instruction::Le, false),
            ast::InfixOperatorKind::Leq => (Instruction::Leq, false),
            ast::InfixOperatorKind::Ge => (Instruction::Le, true),
            ast::InfixOperatorKind::Geq => (Instruction::Leq, true),
            ast::InfixOperatorKind::Eq => (Instruction::Eq, false),
            ast::InfixOperatorKind::Neq => (Instruction::Neq, false),
        };

        if reverse {
            self.compile_node(&node.right)?;
            self.compile_node(&node.left)?;
        } else {
            self.compile_node(&node.left)?;
            self.compile_node(&node.right)?;
        }

        self.emit(instruction, range);

        Ok(())
    }

    fn compile_while(&mut self, while_loop: &ast::While) -> Result<(), Error> {
        let start_index = self.current_scope().instructions.len();
        self.compile_node(&while_loop.condition)?;

        // Jump position will be fixed after
        let jump_index = self.emit(Instruction::JumpNotTruthy(0), while_loop.condition.range);

        self.compile_block(&while_loop.body)?;

        self.emit(Instruction::Jump(start_index), while_loop.body.range);

        let end_index = self.current_scope().instructions.len();
        self.current_scope().instructions[jump_index] = Instruction::JumpNotTruthy(end_index);

        Ok(())
    }

    fn compile_block(&mut self, block: &ast::Block) -> Result<(), Error> {
        for node in &block.nodes {
            self.compile_node(node)?;

            if node.kind() == ast::NodeKind::Expression {
                self.emit(Instruction::Pop, node.range);
            }
        }

        Ok(())
    }

    fn compile_ident(&mut self, ident: &str, range: Range) -> Result<(), Error> {
        let Some(symbol) = self.symbol_table.resolve(ident) else {
            return Err(Error {
                kind: ErrorKind::UndefinedSymbol(ident.to_string()),
                range,
            });
        };

        match symbol {
            Symbol::Global(index) => self.emit(Instruction::LoadGlobal(index), range),
        };

        Ok(())
    }

    fn compile_assign(&mut self, ident: &ast::Node, range: Range) -> Result<(), Error> {
        match &ident.value {
            ast::NodeValue::Identifier(identifier) => {
                let symbol = self.symbol_table.define(identifier.to_string());
                self.compile_store_instruction(symbol, range);
            }
            ast::NodeValue::Index { left, index } => {
                self.compile_node(left)?;
                self.compile_node(index)?;
                self.emit(Instruction::IndexSet, range);
            }
            ast::NodeValue::ArrayLiteral(arr) => {
                self.emit(Instruction::UnpackArray(arr.len()), range);

                for node in arr {
                    self.compile_assign(node, range)?;
                }
            }

            _ => panic!("Invalid asignee: {ident:?}"),
        }

        Ok(())
    }

    fn compile_store_instruction(&mut self, symbol: Symbol, range: Range) {
        match symbol {
            Symbol::Global(index) => self.emit(Instruction::StoreGlobal(index), range),
        };
    }
}
