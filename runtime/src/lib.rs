use parser::ast;

pub mod object;

mod bytecode;
mod compiler;
mod vm;

#[cfg(test)]
mod test;

pub fn run(program: &ast::Program) {
    let compiler = compiler::Compiler::new();
    let bytecode = compiler.compile(program);

    let vm = vm::VirtualMachine::new();
    vm.run(&bytecode);
}
