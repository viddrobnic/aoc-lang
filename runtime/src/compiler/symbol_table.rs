use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Symbol {
    Global(usize),
}

#[derive(Debug)]
pub struct SymbolTable {
    store: HashMap<String, Symbol>,
    num_definitions: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            num_definitions: 0,
        }
    }

    pub fn define(&mut self, name: String) -> Symbol {
        if let Some(symbol) = self.store.get(&name) {
            return *symbol;
        }

        let symbol = Symbol::Global(self.num_definitions);

        self.store.insert(name, symbol);
        self.num_definitions += 1;

        symbol
    }

    pub fn resolve(&self, name: &str) -> Option<Symbol> {
        self.store.get(name).copied()
    }
}
