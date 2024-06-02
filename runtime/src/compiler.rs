use std::rc::Rc;

use crate::{
    bytecode::{Bytecode, Instruction},
    object::Object,
};

use parser::{
    ast::{self, PrefixOperatorKind},
    position::Range,
};

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
            ast::NodeValue::HashLiteral(elements) => self.compile_hash_map(elements, node.range),
            ast::NodeValue::PrefixOperator { .. } => self.compile_prefix_operator(node),
            ast::NodeValue::InfixOperator { .. } => todo!(),
            ast::NodeValue::Assign { .. } => todo!(),
            ast::NodeValue::Index { .. } => todo!(),
            ast::NodeValue::If(_) => todo!(),
            ast::NodeValue::While { .. } => self.compile_while(node),
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

    fn compile_hash_map(&mut self, elements: &[ast::HashLiteralPair], range: Range) {
        for elt in elements {
            self.compile_node(&elt.key);
            self.compile_node(&elt.value);
        }

        self.emit(Instruction::HashMap(elements.len() * 2), range);
    }

    fn compile_prefix_operator(&mut self, node: &ast::Node) {
        let ast::NodeValue::PrefixOperator { operator, right } = &node.value else {
            panic!("Expected prefix operator node, got: {node:?}");
        };

        self.compile_node(right);

        match operator {
            PrefixOperatorKind::Not => self.emit(Instruction::Bang, node.range),
            PrefixOperatorKind::Negative => self.emit(Instruction::Minus, node.range),
        };
    }

    fn compile_while(&mut self, node: &ast::Node) {
        let ast::NodeValue::While { condition, body } = &node.value else {
            panic!("Expected while node, got: {node:?}");
        };

        let start_index = self.current_scope().instructions.len();
        self.compile_node(condition);

        // Jump position will be fixed after
        let jump_index = self.emit(Instruction::JumpNotTruthy(0), condition.range);

        self.compile_block(body);

        self.emit(Instruction::Jump(start_index), body.range);

        let end_index = self.current_scope().instructions.len();
        self.current_scope().instructions[jump_index] = Instruction::JumpNotTruthy(end_index);
    }

    fn compile_block(&mut self, block: &ast::Block) {
        for node in &block.nodes {
            self.compile_node(node);

            if node.kind() == ast::NodeKind::Expression {
                self.emit(Instruction::Pop, node.range);
            }
        }
    }
}
