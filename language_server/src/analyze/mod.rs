use document_info::{DefinitionInfo, DocumentInfo, ReferencesInfo};
use location::{LocationData, LocationEntry};
use parser::{ast, position::Range};
use symbol_table::SymbolTable;

pub mod document_info;
pub mod location;

mod symbol_table;

pub fn analyze(program: &ast::Program) -> DocumentInfo {
    let analyzer = Analyzer::new();
    analyzer.analyze(program)
}

/// Analyzes symbols for go to definition, references, hover, ...
struct Analyzer {
    symbol_table: SymbolTable,
    document_info: DocumentInfo,
}

impl Analyzer {
    fn new() -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            document_info: DocumentInfo {
                definitions: LocationData::new(),
                references: LocationData::new(),
            },
        }
    }

    fn analyze(mut self, program: &ast::Program) -> DocumentInfo {
        for node in &program.statements {
            self.analyze_node(node);
        }

        self.document_info
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

                for arg in &fn_lit.parameters {
                    self.define_ident(arg.name.to_string(), arg.range);
                }
                self.analyze_block(&fn_lit.body);

                self.symbol_table.leave_scope();
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
                self.define_ident(name.to_string(), ident.range);
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

    fn define_ident(&mut self, ident: String, location: Range) {
        let defined_at = self.symbol_table.define(ident, location);

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
}

#[cfg(test)]
mod test {
    use parser::position::{Position, Range};

    use crate::analyze::{
        location::{LocationData, LocationEntry},
        DefinitionInfo, DocumentInfo, ReferencesInfo,
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

        let mut definitions = LocationData::new();
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

        let mut references = LocationData::new();
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

        assert_eq!(
            doc,
            DocumentInfo {
                definitions,
                references,
            }
        );
    }
}
