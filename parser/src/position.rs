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

    pub fn cmp_range(&self, range: &Range) -> PositionOrdering {
        if self.line < range.start.line {
            return PositionOrdering::Before;
        }

        if self.line == range.start.line && self.character < range.start.character {
            return PositionOrdering::Before;
        }

        if self.line > range.end.line {
            return PositionOrdering::After;
        }

        if self.line == range.end.line && self.character >= range.end.character {
            return PositionOrdering::After;
        }

        PositionOrdering::Inside
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PositionOrdering {
    Before,
    Inside,
    After,
}
