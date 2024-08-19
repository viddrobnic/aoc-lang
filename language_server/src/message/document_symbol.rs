use parser::position::Range;
use serde::{Deserialize, Serialize};

use crate::{analyze::symbol_info, TextDocumentIdentifier};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbolParams {
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbol {
    pub name: String,
    pub kind: u8,
    pub range: Range,
    pub selection_range: Range,
    pub children: Vec<DocumentSymbol>,
}

impl DocumentSymbol {
    pub fn map_tree(tree: &[symbol_info::DocumentSymbol]) -> Vec<DocumentSymbol> {
        tree.iter()
            .filter_map(|sym| {
                let Some(name) = &sym.name else {
                    return None;
                };

                Some(DocumentSymbol {
                    name: name.clone(),
                    kind: sym.kind as u8,
                    range: sym.range,
                    selection_range: sym.name_range,
                    children: DocumentSymbol::map_tree(&sym.children),
                })
            })
            .collect()
    }
}
