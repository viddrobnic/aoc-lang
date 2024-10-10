use serde::{Deserialize, Serialize};

use crate::{
    analyze::symbol_info::{DocumentSymbol, DocumentSymbolKind},
    hover::MarkupContent,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionList {
    pub is_incomplete: bool,
    pub items: Vec<CompletionItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    pub label: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text_format: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<MarkupContent>,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum InsertTextFormat {
    PlainText = 1,
    Snippet = 2,
}

impl From<&DocumentSymbol> for Option<CompletionItem> {
    fn from(sym: &DocumentSymbol) -> Self {
        let Some(name) = &sym.name else { return None };

        let kind = match sym.kind {
            DocumentSymbolKind::Function => CompletionItemKind::Function,
            DocumentSymbolKind::Variable => CompletionItemKind::Variable,
        };

        let (text, format) = match sym.kind {
            DocumentSymbolKind::Function => {
                let mut text = format!("{name}(");

                if let Some(params) = &sym.parameters {
                    let params_str = params
                        .iter()
                        .enumerate()
                        .map(|(i, param)| format!("${{{}:{}}}", i + 1, param))
                        .collect::<Vec<_>>()
                        .join(", ");

                    text.push_str(&params_str);
                } else {
                    text.push_str("$1");
                }

                text.push_str(")$0");

                (text, InsertTextFormat::Snippet)
            }
            DocumentSymbolKind::Variable => (name.clone(), InsertTextFormat::PlainText),
        };

        Some(CompletionItem {
            label: name.clone(),
            kind: Some(kind as i32),
            insert_text: Some(text),
            insert_text_format: Some(format as i32),
            documentation: None,
        })
    }
}
