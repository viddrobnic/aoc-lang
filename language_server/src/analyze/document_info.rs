use parser::position::{Position, PositionOrdering, Range};

use crate::message::completion::CompletionItem;

use super::{location::LocationData, symbol_info::DocumentSymbol};

#[derive(Debug, PartialEq, Eq, Default)]
pub struct DefinitionInfo {
    pub defined_at: Range,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ReferencesInfo {
    pub references: Vec<Range>,
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct DocumentInfo {
    pub definitions: LocationData<DefinitionInfo>,
    pub references: LocationData<ReferencesInfo>,

    pub documentation: LocationData<String>,

    pub symbol_tree: Vec<DocumentSymbol>,
}

impl DocumentInfo {
    pub fn get_definition(&self, position: &Position) -> Option<Range> {
        self.definitions
            .get(position)
            .map(|def| def.entry.defined_at)
    }

    pub fn get_references(&self, position: &Position) -> Option<&Vec<Range>> {
        let def_at = self.get_definition(position)?;
        self.references
            .get(&def_at.start)
            .map(|entry| &entry.entry.references)
    }

    pub fn get_documentation(&self, position: &Position) -> Option<&str> {
        let pos = self
            .get_definition(position)
            .map(|range| range.start)
            // Fallback to given position. A hack to avoid traversing the syntax tree on hover
            // request and compute the builtin function docs on request.
            .unwrap_or(*position);

        self.documentation
            .get(&pos)
            .map(|entry| entry.entry.as_ref())
    }

    pub fn get_completion_items(&self, position: &Position) -> Vec<CompletionItem> {
        let mut items = vec![];
        get_completion_items(position, &self.symbol_tree, &mut items);
        items
    }
}

fn get_completion_items(
    position: &Position,
    symbol_tree: &[DocumentSymbol],
    items: &mut Vec<CompletionItem>,
) {
    for symbol in symbol_tree {
        match position.cmp_range(&symbol.range) {
            PositionOrdering::Before => return,
            PositionOrdering::Inside => {
                if let Some(it) = symbol.into() {
                    items.push(it)
                }

                get_completion_items(position, &symbol.children, items);
            }
            PositionOrdering::After => {
                if let Some(it) = symbol.into() {
                    items.push(it)
                }
            }
        }
    }
}
