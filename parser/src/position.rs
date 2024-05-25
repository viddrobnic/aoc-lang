/// Represents position inside code.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Position {
    /// Line number, starting with 0.
    pub line: usize,
    /// Character offset in line, starting with 0.
    pub character: usize,
}

impl Position {
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}
