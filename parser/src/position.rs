/// Represents position inside code.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 0,
            character: 0,
        }
    }
}
