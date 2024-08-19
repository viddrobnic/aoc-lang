use parser::position::Range;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DocumentSymbolKind {
    Function = 12,
    Variable = 13,
}

#[derive(Debug, PartialEq, Eq)]
pub struct DocumentSymbol {
    pub name: Option<String>,
    pub kind: DocumentSymbolKind,
    /// Range of the symbol name (ie. range of function ident)
    pub name_range: Range,
    /// Range of the symbol scope (ie function name + body)
    pub range: Range,
    pub children: Vec<DocumentSymbol>,
}
