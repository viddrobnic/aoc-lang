use parser::ast;

pub mod compiler;
pub mod error;

mod builtin;
mod bytecode;
mod object;
mod vm;

pub fn run(program: &ast::Program) -> Result<(), error::Error> {
    let compiler = compiler::Compiler::new();
    let bytecode = compiler.compile(program)?;

    let mut vm = vm::VirtualMachine::new();
    vm.run(&bytecode)?;

    Ok(())
}
