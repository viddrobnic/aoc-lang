use parser::ast;

pub mod error;
pub mod object;

mod bytecode;
mod compiler;
mod vm;

#[cfg(test)]
mod test;

pub fn run(program: &ast::Program) -> Result<(), error::Error> {
    let compiler = compiler::Compiler::new();
    let bytecode = compiler.compile(program);

    let vm = vm::VirtualMachine::new();
    vm.run(&bytecode)?;

    Ok(())
}
