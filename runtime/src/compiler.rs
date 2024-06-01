use std::rc::Rc;

use crate::{
    bytecode::{Bytecode, Instruction},
    object::Object,
};

use parser::{ast, position::Range};

#[derive(Debug, Default)]
struct Scope {
    instructions: Vec<Instruction>,
    ranges: Vec<Range>,
}

#[derive(Debug)]
pub struct Compiler {
    constants: Vec<Object>,

    scopes: Vec<Scope>,
    scope_index: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            constants: vec![],
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

    pub fn compile(mut self, program: &ast::Program) -> Bytecode {
        for node in &program.statements {
            self.compile_node(node);

            if node.kind() == ast::NodeKind::Expression {
                self.emit(Instruction::Pop, node.range);
            }
        }

        // If compiler works correctly, we should have one scope.
        let scope = self.scopes.pop().expect("Invalid number of scopes");

        Bytecode {
            constants: self.constants,
            instructions: scope.instructions,
            ranges: scope.ranges,
        }
    }

    fn compile_node(&mut self, node: &ast::Node) {
        match &node.value {
            ast::NodeValue::Identifier(_) => todo!(),
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
            ast::NodeValue::ArrayLiteral(arr) => self.compile_array(arr, node.range),
            ast::NodeValue::HashLiteral(_) => todo!(),
            ast::NodeValue::PrefixOperator { .. } => todo!(),
            ast::NodeValue::InfixOperator { .. } => todo!(),
            ast::NodeValue::Assign { .. } => todo!(),
            ast::NodeValue::Index { .. } => todo!(),
            ast::NodeValue::If(_) => todo!(),
            ast::NodeValue::While { .. } => todo!(),
            ast::NodeValue::For { .. } => todo!(),
            ast::NodeValue::Break => todo!(),
            ast::NodeValue::Continue => todo!(),
            ast::NodeValue::FunctionLiteral { .. } => todo!(),
            ast::NodeValue::FunctionCall { .. } => todo!(),
            ast::NodeValue::Return(_) => todo!(),
            ast::NodeValue::Use(_) => todo!(),
        }
    }

    fn compile_constant(&mut self, constant: Object, range: Range) {
        let const_idx = self.add_constant(constant);
        self.emit(Instruction::Constant(const_idx), range);
    }

    fn compile_array(&mut self, arr: &[ast::Node], range: Range) {
        for node in arr {
            self.compile_node(node);
        }

        self.emit(Instruction::Array(arr.len()), range);
    }
}
