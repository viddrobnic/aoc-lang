use std::collections::HashMap;

use crate::builtin::Builtin;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Symbol {
    Global(usize),
    Local(usize),
    Free(usize),
    CurrentClosure,
    Builtin(Builtin),
}

#[derive(Debug)]
pub struct Scope {
    store: HashMap<String, Symbol>,
    pub num_definitions: usize,
    pub captured: Vec<Symbol>,
}

#[derive(Debug)]
pub struct SymbolTable(Vec<Scope>);

impl SymbolTable {
    pub fn new() -> Self {
        Self(vec![Scope {
            store: HashMap::new(),
            num_definitions: 0,
            captured: vec![],
        }])
    }

    pub fn enter_scope(&mut self) {
        self.0.push(Scope {
            store: HashMap::new(),
            num_definitions: 0,
            captured: vec![],
        })
    }

    pub fn leave_scope(&mut self) -> Scope {
        self.0
            .pop()
            .expect("Symbol table should have at least one store")
    }

    pub fn define_current_closure(&mut self, name: String) -> Symbol {
        let symbol = Symbol::CurrentClosure;

        self.0
            .last_mut()
            .expect("Symbol table should have at least one store")
            .store
            .insert(name, symbol);
        symbol
    }

    pub fn define(&mut self, name: String) -> Symbol {
        let is_global = self.0.len() == 1;

        let store = self
            .0
            .last_mut()
            .expect("Symbol table should have at least one store");

        if let Some(symbol) = store.store.get(&name) {
            if matches!(symbol, Symbol::Global(_) | Symbol::Local(_)) {
                return *symbol;
            }
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

    pub fn resolve(&mut self, name: &str) -> Option<Symbol> {
        self.resolve_at(self.0.len() - 1, name)
    }

    fn resolve_at(&mut self, idx: usize, name: &str) -> Option<Symbol> {
        let store = &self.0[idx];
        match store.store.get(name) {
            Some(sym) => Some(*sym),
            None => {
                if idx == 0 {
                    Builtin::from_ident(name).map(Symbol::Builtin)
                } else {
                    self.resolve_parent(idx, name)
                }
            }
        }
    }

    fn resolve_parent(&mut self, idx: usize, name: &str) -> Option<Symbol> {
        let symbol = self.resolve_at(idx - 1, name);
        match symbol {
            None => None,
            Some(Symbol::Global(_)) | Some(Symbol::Builtin(_)) => symbol,
            Some(sym) => Some(self.define_free(idx, sym, name)),
        }
    }

    fn define_free(&mut self, idx: usize, symbol: Symbol, name: &str) -> Symbol {
        let scope = &mut self.0[idx];
        scope.captured.push(symbol);

        let res = Symbol::Free(scope.captured.len() - 1);
        scope.store.insert(name.to_string(), res);
        res
    }
}

#[cfg(test)]
mod test {
    use crate::{builtin::Builtin, compiler::symbol_table::Symbol};

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

    #[test]
    fn free_scope() {
        let mut table = SymbolTable::new();

        table.define("a".to_string());

        table.enter_scope();

        assert_eq!(table.resolve("a"), Some(Symbol::Global(0)));
        assert_eq!(table.resolve("len"), Some(Symbol::Builtin(Builtin::Len)));

        table.define("b".to_string());
        assert_eq!(table.resolve("b"), Some(Symbol::Local(0)));

        table.enter_scope();
        assert_eq!(table.resolve("a"), Some(Symbol::Global(0)));
        assert_eq!(table.resolve("b"), Some(Symbol::Free(0)));

        table.define("b".to_string());
        assert_eq!(table.resolve("b"), Some(Symbol::Local(0)));
        table.define("c".to_string());
        assert_eq!(table.resolve("c"), Some(Symbol::Local(1)));

        let scope = table.leave_scope();
        assert_eq!(scope.num_definitions, 2);
        assert_eq!(scope.captured, vec![Symbol::Local(0)]);
    }
}
