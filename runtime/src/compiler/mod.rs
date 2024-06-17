use std::{fs::File, io::Read, mem, rc::Rc};

use crate::{
    bytecode::{Bytecode, CreateClosure, Function, Instruction},
    error::{Error, ErrorKind},
    object::Object,
};

use parser::{ast, position::Range};

use self::symbol_table::{Symbol, SymbolTable};

mod symbol_table;

#[cfg(test)]
mod test;

#[derive(Debug)]
struct LoopInfo {
    // Indices of break instructions
    breaks: Vec<usize>,
    // Indices of continue instructions
    continues: Vec<usize>,
}

#[derive(Debug, Default)]
struct Scope {
    instructions: Vec<Instruction>,
    ranges: Vec<Range>,

    loops: Vec<LoopInfo>,
}

impl Scope {
    fn enter_loop(&mut self) {
        self.loops.push(LoopInfo {
            breaks: vec![],
            continues: vec![],
        })
    }

    fn exit_loop(&mut self) -> Option<LoopInfo> {
        self.loops.pop()
    }
}

#[derive(Debug)]
pub struct Compiler {
    constants: Vec<Object>,
    functions: Vec<Function>,

    symbol_table: SymbolTable,

    scopes: Vec<Scope>,
    scope_index: usize,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            constants: vec![],
            functions: vec![],
            symbol_table: SymbolTable::new(),
            scopes: vec![Scope::default()],
            scope_index: 0,
        }
    }

    fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
        self.scope_index += 1;

        self.symbol_table.enter_scope();
    }

    fn exist_scope(&mut self) -> (Scope, symbol_table::Scope) {
        let scope = self.scopes.pop().expect("Exiting on nos scopes");
        self.scope_index -= 1;

        let sym_scope = self.symbol_table.leave_scope();
        (scope, sym_scope)
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
        let main_fn = Function {
            instructions: scope.instructions,
            ranges: scope.ranges,
            nr_local_variables: 0,
            nr_arguments: 0,
        };
        self.functions.push(main_fn);
        let main_fn_idx = self.functions.len() - 1;

        Ok(Bytecode {
            constants: self.constants,
            functions: self.functions,
            main_function: main_fn_idx,
        })
    }

    fn compile_node(&mut self, node: &ast::Node) -> Result<(), Error> {
        match &node.value {
            ast::NodeValue::Identifier(ident) => self.compile_ident(ident, node.range)?,
            ast::NodeValue::Null => self.compile_constant(Object::Null, node.range),
            ast::NodeValue::IntegerLiteral(int) => {
                self.compile_constant(Object::Integer(*int), node.range);
            }
            ast::NodeValue::FloatLiteral(flt) => {
                self.compile_constant(Object::Float(*flt), node.range);
            }
            ast::NodeValue::CharLiteral(ch) => self.compile_constant(Object::Char(*ch), node.range),
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
            ast::NodeValue::Assign(assign) => {
                self.compile_node(&assign.value)?;
                self.compile_assign(&assign.ident, node.range)?;
            }
            ast::NodeValue::Index(index) => {
                self.compile_node(&index.left)?;
                self.compile_node(&index.index)?;
                self.emit(Instruction::IndexGet, node.range);
            }
            ast::NodeValue::If(if_node) => self.compile_if(if_node)?,
            ast::NodeValue::While(while_loop) => self.compile_while(while_loop)?,
            ast::NodeValue::For(for_loop) => self.compile_for(for_loop)?,
            ast::NodeValue::Break => self.compile_break(node.range)?,
            ast::NodeValue::Continue => self.compile_continue(node.range)?,
            ast::NodeValue::FunctionLiteral(fn_literal) => {
                self.compile_fn_literal(fn_literal, node.range)?;
            }
            ast::NodeValue::FunctionCall(fn_call) => self.compile_fn_call(fn_call, node.range)?,
            ast::NodeValue::Return(ret_node) => self.compile_return(ret_node, node.range)?,
            ast::NodeValue::Use(path) => self.compile_use(path, node.range)?,
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
            ast::PrefixOperatorKind::Not => self.emit(Instruction::Bang, range),
            ast::PrefixOperatorKind::Negative => self.emit(Instruction::Minus, range),
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

    fn compile_if(&mut self, if_node: &ast::IfNode) -> Result<(), Error> {
        self.compile_node(&if_node.condition)?;

        // Jump to alternative
        let jump_cons = self.emit(Instruction::JumpNotTruthy(0), if_node.condition.range);

        // Compile consequence and add jump to skip alternative
        self.compile_block(&if_node.consequence, true)?;
        let jump_alt = self.emit(Instruction::Jump(0), if_node.consequence.range);

        // Fix jump after consequence index.
        let cons_index = self.current_scope().instructions.len();
        self.current_scope().instructions[jump_cons] = Instruction::JumpNotTruthy(cons_index);

        match &if_node.alternative {
            Some(alt) => self.compile_block(alt, true)?,
            None => {
                self.emit(Instruction::Null, if_node.consequence.range);
            }
        }

        let alt_index = self.current_scope().instructions.len();
        self.current_scope().instructions[jump_alt] = Instruction::Jump(alt_index);

        Ok(())
    }

    fn compile_while(&mut self, while_loop: &ast::While) -> Result<(), Error> {
        let start_index = self.current_scope().instructions.len();
        self.current_scope().enter_loop();

        self.compile_node(&while_loop.condition)?;

        // Jump position will be fixed after
        let jump_index = self.emit(Instruction::JumpNotTruthy(0), while_loop.condition.range);

        self.compile_block(&while_loop.body, false)?;

        self.emit(Instruction::Jump(start_index), while_loop.body.range);

        let end_index = self.current_scope().instructions.len();
        self.current_scope().instructions[jump_index] = Instruction::JumpNotTruthy(end_index);

        // We entered the loop, so it's safe to unwrap.
        let loop_info = self.current_scope().exit_loop().unwrap();
        for break_idx in &loop_info.breaks {
            self.current_scope().instructions[*break_idx] = Instruction::Jump(end_index);
        }
        for continue_idx in &loop_info.continues {
            self.current_scope().instructions[*continue_idx] = Instruction::Jump(start_index);
        }

        Ok(())
    }

    fn compile_for(&mut self, for_loop: &ast::For) -> Result<(), Error> {
        self.compile_node(&for_loop.initial)?;

        let start_index = self.current_scope().instructions.len();
        self.current_scope().enter_loop();

        self.compile_node(&for_loop.condition)?;

        // Jump position will be fixed after
        let jump_index = self.emit(Instruction::JumpNotTruthy(0), for_loop.condition.range);

        // Compile the body
        self.compile_block(&for_loop.body, false)?;

        let after_index = self.current_scope().instructions.len();
        self.compile_node(&for_loop.after)?;
        if for_loop.after.kind() == ast::NodeKind::Expression {
            self.emit(Instruction::Pop, for_loop.after.range);
        }

        self.emit(Instruction::Jump(start_index), for_loop.body.range);

        let end_index = self.current_scope().instructions.len();
        self.current_scope().instructions[jump_index] = Instruction::JumpNotTruthy(end_index);

        // We entered the loop, so it's safe to unwrap.
        let loop_info = self.current_scope().exit_loop().unwrap();
        for break_idx in loop_info.breaks {
            self.current_scope().instructions[break_idx] = Instruction::Jump(end_index);
        }
        for continue_idx in loop_info.continues {
            self.current_scope().instructions[continue_idx] = Instruction::Jump(after_index);
        }

        Ok(())
    }

    // Compiles block. If emit_last is true, last statement in the block will be left on stack.
    // In case value was not pushed in the last node of the block, null will be pushed.
    fn compile_block(&mut self, block: &ast::Block, emit_last: bool) -> Result<(), Error> {
        if emit_last && block.nodes.is_empty() {
            self.emit(Instruction::Null, block.range);
            return Ok(());
        }

        for node in &block.nodes {
            self.compile_node(node)?;

            if node.kind() == ast::NodeKind::Expression {
                self.emit(Instruction::Pop, node.range);
            }
        }

        if !emit_last {
            return Ok(());
        }

        // We already handled empty block where emit last is true, so it's safe to unwrap.
        let last = block.nodes.last().unwrap();
        match last.kind() {
            ast::NodeKind::Expression => {
                // Remove the `pop` instruction
                self.current_scope().instructions.pop();
                self.current_scope().ranges.pop();
            }
            ast::NodeKind::Statement => {
                self.emit(Instruction::Null, last.range);
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

        self.compile_load_instruction(symbol, range);
        Ok(())
    }

    fn compile_assign(&mut self, ident: &ast::Node, range: Range) -> Result<(), Error> {
        match &ident.value {
            ast::NodeValue::Identifier(identifier) => {
                let symbol = self.symbol_table.define(identifier.to_string());
                self.compile_store_instruction(symbol, range);
            }
            ast::NodeValue::Index(index) => {
                self.compile_node(&index.left)?;
                self.compile_node(&index.index)?;
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

    fn compile_break(&mut self, range: Range) -> Result<(), Error> {
        let idx = self.current_scope().instructions.len();
        let Some(loop_info) = self.current_scope().loops.last_mut() else {
            return Err(Error {
                kind: ErrorKind::ControlFlowOutsideOfLoop,
                range,
            });
        };
        loop_info.breaks.push(idx);

        // Jump index will be fixed in compile loop function.
        self.emit(Instruction::Jump(0), range);

        Ok(())
    }

    fn compile_continue(&mut self, range: Range) -> Result<(), Error> {
        let idx = self.current_scope().instructions.len();
        let Some(loop_info) = self.current_scope().loops.last_mut() else {
            return Err(Error {
                kind: ErrorKind::ControlFlowOutsideOfLoop,
                range,
            });
        };
        loop_info.continues.push(idx);

        self.emit(Instruction::Jump(0), range);

        Ok(())
    }

    fn compile_return(&mut self, node: &ast::Node, range: Range) -> Result<(), Error> {
        if self.scope_index == 0 {
            return Err(Error {
                kind: ErrorKind::ReturnOutsideOfFunction,
                range,
            });
        }

        self.compile_node(node)?;
        self.emit(Instruction::Return, range);

        Ok(())
    }

    fn compile_fn_literal(
        &mut self,
        fn_literal: &ast::FunctionLiteral,
        range: Range,
    ) -> Result<(), Error> {
        self.enter_scope();

        if let Some(name) = &fn_literal.name {
            self.symbol_table.define_current_closure(name.to_string());
        }

        // Define argument symbols
        for param in &fn_literal.parameters {
            self.symbol_table.define(param.to_string());
        }

        // Compile body
        self.compile_block(&fn_literal.body, true)?;
        self.emit(Instruction::Return, fn_literal.body.range);

        // Exit scope
        let (scope, sym_scope) = self.exist_scope();

        // Create function
        let func = Function {
            instructions: scope.instructions,
            ranges: scope.ranges,
            nr_local_variables: sym_scope.num_definitions,
            nr_arguments: fn_literal.parameters.len(),
        };
        self.functions.push(func);

        // Push captured on stack for creating closure
        for sym in &sym_scope.captured {
            self.compile_load_instruction(*sym, range)
        }

        // Create closure
        self.emit(
            Instruction::CreateClosure(CreateClosure {
                function_index: self.functions.len() - 1,
                nr_free_variables: sym_scope.captured.len(),
            }),
            range,
        );

        Ok(())
    }

    fn compile_fn_call(&mut self, fn_call: &ast::FunctionCall, range: Range) -> Result<(), Error> {
        for arg in &fn_call.arguments {
            self.compile_node(arg)?;
        }

        self.compile_node(&fn_call.function)?;

        self.emit(Instruction::FnCall(fn_call.arguments.len()), range);
        Ok(())
    }

    fn compile_use(&mut self, path: &str, range: Range) -> Result<(), Error> {
        // Read file
        let mut file = File::open(path).map_err(|_| Error {
            kind: ErrorKind::InvalidImportPath(path.to_string()),
            range,
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|_| Error {
            kind: ErrorKind::InvalidImportPath(path.to_string()),
            range,
        })?;

        // Parse
        let program = parser::parse(&content).map_err(|err| Error {
            kind: ErrorKind::ImportParserError {
                path: path.to_string(),
                error: err,
            },
            range,
        })?;

        // Transform to function call
        let import = ast::FunctionCall {
            function: Box::new(ast::Node {
                value: ast::NodeValue::FunctionLiteral(ast::FunctionLiteral {
                    name: None,
                    parameters: vec![],
                    body: ast::Block {
                        nodes: program.statements,
                        range,
                    },
                }),
                range,
            }),
            arguments: vec![],
        };

        // Swap symbol table with empty symbol table, to avoid using
        // globals in current file and capturing variables from current
        // file in imported file.
        let mut sym_table = SymbolTable::new();
        mem::swap(&mut self.symbol_table, &mut sym_table);

        // Compile the call
        self.compile_fn_call(&import, range).map_err(|err| Error {
            kind: ErrorKind::ImportCompilerError {
                path: path.to_string(),
                error: Box::new(err),
            },
            range,
        })?;

        // Swap symbol table back
        mem::swap(&mut self.symbol_table, &mut sym_table);

        Ok(())
    }

    fn compile_store_instruction(&mut self, symbol: Symbol, range: Range) {
        match symbol {
            Symbol::Global(index) => self.emit(Instruction::StoreGlobal(index), range),
            Symbol::Local(index) => self.emit(Instruction::StoreLocal(index), range),
            Symbol::Free(_) => panic!("Can't store to free symbol"),
            Symbol::CurrentClosure => panic!("Can't store to current closure symbol"),
            Symbol::Builtin(_) => panic!("Can't store to builtin"),
        };
    }

    fn compile_load_instruction(&mut self, symbol: Symbol, range: Range) {
        match symbol {
            Symbol::Global(index) => self.emit(Instruction::LoadGlobal(index), range),
            Symbol::Local(index) => self.emit(Instruction::LoadLocal(index), range),
            Symbol::Free(index) => self.emit(Instruction::LoadFree(index), range),
            Symbol::CurrentClosure => self.emit(Instruction::CurrentClosure, range),
            Symbol::Builtin(bltin) => self.emit(Instruction::Builtin(bltin), range),
        };
    }
}
