use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Symbol {
    Global(usize),
    Local(usize),
}

#[derive(Debug)]
pub struct Scope {
    store: HashMap<String, Symbol>,
    pub num_definitions: usize,
}

#[derive(Debug)]
pub struct SymbolTable(Vec<Scope>);

impl SymbolTable {
    pub fn new() -> Self {
        Self(vec![Scope {
            store: HashMap::new(),
            num_definitions: 0,
        }])
    }

    pub fn enter_scope(&mut self) {
        self.0.push(Scope {
            store: HashMap::new(),
            num_definitions: 0,
        })
    }

    pub fn leave_scope(&mut self) -> Scope {
        self.0
            .pop()
            .expect("Symbol table should have at least one store")
    }

    pub fn define(&mut self, name: String) -> Symbol {
        let is_global = self.0.len() == 1;

        let store = self
            .0
            .last_mut()
            .expect("Symbol table should have at least one store");

        if let Some(symbol) = store.store.get(&name) {
            return *symbol;
        }

        let symbol = if is_global {
            Symbol::Global(store.num_definitions)
        } else {
            Symbol::Local(store.num_definitions)
        };

        store.store.insert(name, symbol);
        store.num_definitions += 1;

        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        self.resolve_at(self.0.len() - 1, name)
    }

    fn resolve_at(&self, idx: usize, name: &str) -> Option<Symbol> {
        let store = &self.0[idx];
        match store.store.get(name) {
            Some(sym) => Some(*sym),
            None => {
                if idx == 0 {
                    None
                } else {
                    self.resolve_at(idx - 1, name)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::compiler::symbol_table::Symbol;

    use super::SymbolTable;

    #[test]
    fn local_scope() {
        let mut table = SymbolTable::new();

        // Define global.
        table.define("a".to_string());

        table.enter_scope();

        // Get global
        let sym = table.resolve("a");
        assert_eq!(sym, Some(Symbol::Global(0)));
        // Get undefined
        assert_eq!(table.resolve("b"), None);

        // shadow a, define b
        table.define("a".to_string());
        table.define("b".to_string());
        assert_eq!(table.resolve("a"), Some(Symbol::Local(0)));
        assert_eq!(table.resolve("b"), Some(Symbol::Local(1)));

        // Exit local
        table.leave_scope();
        assert_eq!(table.resolve("a"), Some(Symbol::Global(0)));
        assert_eq!(table.resolve("b"), None);
    }
}
