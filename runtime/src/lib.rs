pub mod object;

mod bytecode;
mod compiler;

#[cfg(test)]
mod test;

pub fn run(input: &str) {
    // TODO: Haandle errors and add vm
    let program = parser::parse(input).unwrap();

    let compiler = compiler::Compiler::new();
    let _bytecode = compiler.compile(&program);
}
