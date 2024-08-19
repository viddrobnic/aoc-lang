use parser::position::{Position, Range};

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

    // TODO NOTE:
    // Document symbols hierarchy are gotten by just recursively mapping the vec of document symbols + filtering out not named symbols.
    //
    // Autocomplete is implemented by iterating the vec of symbols:
    // - symbol is before current position: add the name to possible autocomplete values
    // - current position is inside symbol: add the name of the possible autocomplete values and
    //   recursively call autocomplete on children of the symbol
    // - symbol is after current position: exit the loop
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
}
