use parser::ast;

pub mod error;

mod bytecode;
mod compiler;
mod object;
mod vm;

pub fn run(program: &ast::Program) -> Result<(), error::Error> {
    let compiler = compiler::Compiler::new();
    let bytecode = compiler.compile(program)?;

    let mut vm = vm::VirtualMachine::new();
    vm.run(&bytecode)?;

    Ok(())
}
