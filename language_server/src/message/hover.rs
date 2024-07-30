use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Hover {
    pub contents: MarkupContent,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkupKind {
    PlainText,
    Markdown,
}
