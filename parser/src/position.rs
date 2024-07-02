/// Represents position inside code.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Position {
    /// Line number, starting with 0.
    pub line: usize,
    /// Character offsets count UTF-16 code units.
    pub character: usize,
}

impl Position {
    pub fn new(line: usize, character: usize) -> Self {
        Self { line, character }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Range {
    /// Range start position, inclusive.
    pub start: Position,

    /// Range end position, exclusive.
    /// To represent range that contains eol, set end position
    /// to start of next line.
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}
