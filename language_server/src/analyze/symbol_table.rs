use std::collections::HashMap;

use parser::position::Range;

#[derive(Debug)]
pub struct SymbolTable(Vec<HashMap<String, Range>>);

impl SymbolTable {
    pub fn new() -> Self {
        Self(vec![HashMap::new()])
    }

    pub fn enter_scope(&mut self) {
        self.0.push(HashMap::new());
    }

    pub fn leave_scope(&mut self) {
        self.0.pop();
    }

    /// Define symbol in current scope. Returns the range
    /// of where the symbol was first defined.
    pub fn define(&mut self, name: String, location: Range) -> Range {
        let store = self
            .0
            .last_mut()
            .expect("Symbol table should have at leas one store");

        if let Some(sym_loc) = store.get(&name) {
            return *sym_loc;
        }

        store.insert(name, location);
        location
    }

    /// Resolves the symbol and returns the range
    /// of where it was defined.
    pub fn resolve(&self, name: &str) -> Option<Range> {
        for scope in self.0.iter().rev() {
            if let Some(range) = scope.get(name) {
                return Some(*range);
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use parser::position::{Position, Range};

    use super::SymbolTable;

    #[test]
    fn symbol_table() {
        let mut table = SymbolTable::new();

        let range1 = Range {
            start: Position::new(0, 0),
            end: Position::new(1, 0),
        };

        let range2 = Range {
            start: Position::new(2, 0),
            end: Position::new(2, 10),
        };

        let rng = table.define("a".to_string(), range1);
        assert_eq!(rng, range1);
        assert_eq!(table.resolve("a"), Some(range1));

        // redefine
        let rng = table.define("a".to_string(), range2);
        assert_eq!(rng, range1);
        assert_eq!(table.resolve("a"), Some(range1));

        // new scope
        table.enter_scope();
        assert_eq!(table.resolve("a"), Some(range1));

        // redefine in new scope
        let rng = table.define("a".to_string(), range2);
        assert_eq!(rng, range2);
        assert_eq!(table.resolve("a"), Some(range2));

        // exit scope
        table.leave_scope();
        assert_eq!(table.resolve("a"), Some(range1));
    }
}
