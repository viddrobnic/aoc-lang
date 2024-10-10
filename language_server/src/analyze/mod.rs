use document_info::{DefinitionInfo, DocumentInfo, ReferencesInfo};
use documentation::make_documentation_location_data;
use location::{LocationData, LocationEntry};
use parser::{
    ast,
    position::{Position, Range},
};
use runtime::builtin::Builtin;
use symbol_info::{DocumentSymbol, DocumentSymbolKind};
use symbol_table::SymbolTable;

pub mod document_info;
pub mod location;
pub mod symbol_info;

mod documentation;
mod symbol_table;

pub fn analyze(program: &ast::Program) -> DocumentInfo {
    let analyzer = Analyzer::new();
    analyzer.analyze(program)
}

/// Analyzes symbols for go to definition, references, hover, ...
struct Analyzer {
    symbol_table: SymbolTable,
    documentation: LocationData<String>,
    symbols: Vec<Vec<DocumentSymbol>>,

    document_info: DocumentInfo,
}

impl Analyzer {
    fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            documentation: LocationData::default(),
            symbols: vec![vec![]],
            document_info: DocumentInfo::default(),
        }
    }

    fn analyze(mut self, program: &ast::Program) -> DocumentInfo {
        self.documentation = make_documentation_location_data(&program.comments);

        for node in &program.statements {
            self.analyze_node(node);
        }

        let mut res = self.document_info;
        res.symbol_tree = self.symbols.pop().unwrap();
        res
    }

    fn analyze_node(&mut self, node: &ast::Node) {
        match &node.value {
            ast::NodeValue::Identifier(ident) => {
                self.resolve_ident(ident, node.range);
            }
            ast::NodeValue::ArrayLiteral(arr) => {
                for arr_node in arr {
                    self.analyze_node(arr_node);
                }
            }
            ast::NodeValue::HashLiteral(pairs) => {
                for pair in pairs {
                    self.analyze_node(&pair.key);
                    self.analyze_node(&pair.value);
                }
            }
            ast::NodeValue::PrefixOperator(prefix) => self.analyze_node(&prefix.right),
            ast::NodeValue::InfixOperator(infix) => {
                self.analyze_node(&infix.left);
                self.analyze_node(&infix.right);
            }
            ast::NodeValue::Assign(assign) => {
                self.analyze_assign(&assign.ident);
                self.analyze_node(&assign.value);
            }
            ast::NodeValue::Index(index) => {
                self.analyze_node(&index.left);
                self.analyze_node(&index.index);
            }
            ast::NodeValue::If(if_node) => {
                self.analyze_node(&if_node.condition);
                self.analyze_block(&if_node.consequence);
                if let Some(alt) = &if_node.alternative {
                    self.analyze_block(alt);
                }
            }
            ast::NodeValue::While(while_node) => {
                self.analyze_node(&while_node.condition);
                self.analyze_block(&while_node.body);
            }
            ast::NodeValue::For(for_node) => {
                self.analyze_node(&for_node.initial);
                self.analyze_node(&for_node.condition);
                self.analyze_node(&for_node.after);
                self.analyze_block(&for_node.body);
            }
            ast::NodeValue::Return(ret) => self.analyze_node(ret),
            ast::NodeValue::FunctionLiteral(fn_lit) => {
                self.symbol_table.enter_scope();
                self.symbols.push(vec![]);

                for arg in &fn_lit.parameters {
                    self.define_ident(arg.name.to_string(), arg.range, false);
                }
                self.analyze_block(&fn_lit.body);

                self.symbol_table.leave_scope();

                let children = self.symbols.pop().unwrap();
                let parameters: Vec<_> = fn_lit
                    .parameters
                    .iter()
                    .map(|param| param.name.clone())
                    .collect();

                if fn_lit.name.is_some() {
                    // The function is named, which means that the last symbol on top of the
                    // frame is the name of the function. We have to update the range, kind and children.
                    let fn_sym = self.symbols.last_mut().unwrap().last_mut().unwrap();
                    fn_sym.range.end = node.range.end;
                    fn_sym.kind = DocumentSymbolKind::Function;
                    fn_sym.parameters = Some(parameters);
                    fn_sym.children = children;
                } else {
                    // The function is not named => it is anonymous.
                    // We add it on top of the current frame as anonymous symbol.
                    self.symbols.last_mut().unwrap().push(DocumentSymbol {
                        name: None,
                        kind: DocumentSymbolKind::Function,
                        parameters: Some(parameters),
                        name_range: node.range,
                        range: node.range,
                        children,
                    });
                }
            }
            ast::NodeValue::FunctionCall(fn_call) => {
                self.analyze_node(&fn_call.function);
                for arg in &fn_call.arguments {
                    self.analyze_node(arg);
                }
            }
            ast::NodeValue::Break => (),
            ast::NodeValue::Continue => (),
            ast::NodeValue::Use(_) => (),
            ast::NodeValue::Null => (),
            ast::NodeValue::IntegerLiteral(_) => (),
            ast::NodeValue::FloatLiteral(_) => (),
            ast::NodeValue::CharLiteral(_) => (),
            ast::NodeValue::BoolLiteral(_) => (),
            ast::NodeValue::StringLiteral(_) => (),
        }
    }

    fn analyze_block(&mut self, block: &ast::Block) {
        for node in &block.nodes {
            self.analyze_node(node);
        }
    }

    fn analyze_assign(&mut self, ident: &ast::Node) {
        match &ident.value {
            ast::NodeValue::Identifier(name) => {
                self.define_ident(name.to_string(), ident.range, true);
            }
            ast::NodeValue::Index(index) => {
                self.analyze_node(&index.left);
                self.analyze_node(&index.index);
            }
            ast::NodeValue::ArrayLiteral(arr) => {
                for node in arr {
                    self.analyze_assign(node);
                }
            }

            _ => panic!("Invalid asignee: {ident:?}"),
        }
    }

    fn resolve_ident(&mut self, ident: &str, location: Range) {
        let Some(defined_at) = self.symbol_table.resolve(ident) else {
            // The ident has not yet been defined. If we are using a ident
            // that is not defined, it's probably a builtin function. Add documentation for builtin
            // function at this poistion.
            //
            // If we have a lot of builtin function calls, we will copy the documentation for each call.
            // This could be optimized by dynamicaly checking if request hover position
            // is inside builtin function (in the textDocument/hover handler).
            // I won't do this (for now), because it's a toy language and there is a lot
            // of other things I want to try, before starting with optimizations :)
            self.define_builtin_documentation(ident, location);
            return;
        };

        // We are scanning the ast from top to bottom, which means that
        // location should be strictly increasing, and it's fine to unwrap.

        // Set definition
        self.document_info
            .definitions
            .push(LocationEntry {
                location,
                entry: DefinitionInfo { defined_at },
            })
            .unwrap();

        // Add reference. It's fine to unwrap, since reference should be set.
        let references = self
            .document_info
            .references
            .get_mut(&defined_at.start)
            .unwrap();
        references.entry.references.push(location);
    }

    // Adds definition to document info. Documentation is only added if node is part
    // of an ast::Asign. In this case assign_node should be Some(_). If assign_node is None,
    // no documentation info is added.
    fn define_ident(&mut self, ident: String, location: Range, define_documentation: bool) {
        let defined_at = self.symbol_table.define(ident.clone(), location);

        // We are scanning the ast from top to bottom, which means that
        // location should be strictly increasing, and it's fine to unwrap.

        // Set definition
        self.document_info
            .definitions
            .push(LocationEntry {
                location,
                entry: DefinitionInfo { defined_at },
            })
            .unwrap();

        let refs = &mut self.document_info.references;

        if defined_at == location {
            // The symbol is newly defined (location is the same at where it's defined).
            // We create a new references entry
            refs.push(LocationEntry {
                location,
                entry: ReferencesInfo {
                    references: vec![location],
                },
            })
            .unwrap();

            // Add to documentation info if needed.
            if define_documentation {
                self.define_documentation(location);
            }

            // Add document symbol to tree of symbols
            self.symbols.last_mut().unwrap().push(DocumentSymbol {
                name: Some(ident),
                kind: DocumentSymbolKind::Variable,
                parameters: None,
                name_range: location,
                range: location,
                children: vec![],
            })
        } else {
            // The symbol is already defined, we are just mutating it.
            // We update the existing references entry.
            refs.get_mut(&defined_at.start)
                .unwrap()
                .entry
                .references
                .push(location);
        }
    }

    fn define_documentation(&mut self, location: Range) {
        if location.start.line == 0 {
            return;
        }

        let doc_pos = Position {
            line: location.start.line - 1,
            character: 0,
        };

        let Some(documentation) = self.documentation.get(&doc_pos) else {
            return;
        };

        self.document_info
            .documentation
            .push(LocationEntry {
                location,
                entry: documentation.entry.clone(),
            })
            .unwrap();
    }

    fn define_builtin_documentation(&mut self, ident: &str, location: Range) {
        let Some(builtin) = Builtin::from_ident(ident) else {
            return;
        };

        self.document_info
            .documentation
            .push(LocationEntry {
                location,
                entry: builtin.documentation(),
            })
            .unwrap();
    }
}

#[cfg(test)]
mod test {
    use parser::position::{Position, Range};

    use crate::analyze::{
        location::{LocationData, LocationEntry},
        symbol_info::{DocumentSymbol, DocumentSymbolKind},
        DefinitionInfo, ReferencesInfo,
    };

    use super::analyze;

    #[test]
    fn symbol_info() {
        let input = r#"
            a = 10
            [a, b, foo.bar, [c]] = [1, 2, 3, [4]]
            c
            "#;
        let program = parser::parse(input).unwrap();
        let doc = analyze(&program);

        let a_range = Range {
            start: Position::new(1, 12),
            end: Position::new(1, 13),
        };
        let b_range = Range {
            start: Position::new(2, 16),
            end: Position::new(2, 17),
        };
        let c_range = Range {
            start: Position::new(2, 29),
            end: Position::new(2, 30),
        };

        let mut definitions = LocationData::default();
        definitions
            .push(LocationEntry {
                location: a_range,
                entry: DefinitionInfo {
                    defined_at: a_range,
                },
            })
            .unwrap();
        definitions
            .push(LocationEntry {
                location: Range {
                    start: Position::new(2, 13),
                    end: Position::new(2, 14),
                },
                entry: DefinitionInfo {
                    defined_at: a_range,
                },
            })
            .unwrap();
        definitions
            .push(LocationEntry {
                location: b_range,
                entry: DefinitionInfo {
                    defined_at: b_range,
                },
            })
            .unwrap();
        definitions
            .push(LocationEntry {
                location: c_range,
                entry: DefinitionInfo {
                    defined_at: c_range,
                },
            })
            .unwrap();
        definitions
            .push(LocationEntry {
                location: Range {
                    start: Position::new(3, 12),
                    end: Position::new(3, 13),
                },
                entry: DefinitionInfo {
                    defined_at: c_range,
                },
            })
            .unwrap();

        let mut references = LocationData::default();
        references
            .push(LocationEntry {
                location: a_range,
                entry: ReferencesInfo {
                    references: vec![
                        a_range,
                        Range {
                            start: Position::new(2, 13),
                            end: Position::new(2, 14),
                        },
                    ],
                },
            })
            .unwrap();
        references
            .push(LocationEntry {
                location: b_range,
                entry: ReferencesInfo {
                    references: vec![b_range],
                },
            })
            .unwrap();
        references
            .push(LocationEntry {
                location: c_range,
                entry: ReferencesInfo {
                    references: vec![
                        c_range,
                        Range {
                            start: Position::new(3, 12),
                            end: Position::new(3, 13),
                        },
                    ],
                },
            })
            .unwrap();

        assert_eq!(doc.definitions, definitions);
        assert_eq!(doc.references, references);
    }

    #[test]
    fn symbol_tree() {
        let input = r#"
            a = 10

            fn() {
                a = 20
                b = 30
            }

            a = 20

            foo = fn(bar) {
                [a, b] = [2, 3]
            }
            "#;
        let program = parser::parse(input).unwrap();
        let doc = analyze(&program);

        assert_eq!(
            doc.symbol_tree,
            vec![
                DocumentSymbol {
                    name: Some("a".to_string()),
                    kind: DocumentSymbolKind::Variable,
                    parameters: None,
                    name_range: Range::new(Position::new(1, 12), Position::new(1, 13)),
                    range: Range::new(Position::new(1, 12), Position::new(1, 13)),
                    children: vec![],
                },
                DocumentSymbol {
                    name: None,
                    kind: DocumentSymbolKind::Function,
                    parameters: Some(vec![]),
                    name_range: Range::new(Position::new(3, 12), Position::new(6, 13)),
                    range: Range::new(Position::new(3, 12), Position::new(6, 13)),
                    children: vec![
                        DocumentSymbol {
                            name: Some("a".to_string()),
                            kind: DocumentSymbolKind::Variable,
                            parameters: None,
                            name_range: Range::new(Position::new(4, 16), Position::new(4, 17)),
                            range: Range::new(Position::new(4, 16), Position::new(4, 17)),
                            children: vec![],
                        },
                        DocumentSymbol {
                            name: Some("b".to_string()),
                            kind: DocumentSymbolKind::Variable,
                            parameters: None,
                            name_range: Range::new(Position::new(5, 16), Position::new(5, 17)),
                            range: Range::new(Position::new(5, 16), Position::new(5, 17)),
                            children: vec![],
                        }
                    ]
                },
                DocumentSymbol {
                    name: Some("foo".to_string()),
                    kind: DocumentSymbolKind::Function,
                    parameters: Some(vec!["bar".to_string()]),
                    name_range: Range::new(Position::new(10, 12), Position::new(10, 15)),
                    range: Range::new(Position::new(10, 12), Position::new(12, 13)),
                    children: vec![
                        DocumentSymbol {
                            name: Some("bar".to_string()),
                            kind: DocumentSymbolKind::Variable,
                            parameters: None,
                            name_range: Range::new(Position::new(10, 21), Position::new(10, 24)),
                            range: Range::new(Position::new(10, 21), Position::new(10, 24)),
                            children: vec![],
                        },
                        DocumentSymbol {
                            name: Some("a".to_string()),
                            kind: DocumentSymbolKind::Variable,
                            parameters: None,
                            name_range: Range::new(Position::new(11, 17), Position::new(11, 18)),
                            range: Range::new(Position::new(11, 17), Position::new(11, 18)),
                            children: vec![],
                        },
                        DocumentSymbol {
                            name: Some("b".to_string()),
                            kind: DocumentSymbolKind::Variable,
                            parameters: None,
                            name_range: Range::new(Position::new(11, 20), Position::new(11, 21)),
                            range: Range::new(Position::new(11, 20), Position::new(11, 21)),
                            children: vec![],
                        }
                    ]
                },
            ]
        );
    }
}
