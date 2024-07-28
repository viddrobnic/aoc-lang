use serde::{Deserialize, Serialize};

use crate::TextDocumentPositionParams;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReferenceParams {
    #[serde(flatten)]
    pub text_position: TextDocumentPositionParams,

    pub context: ReferenceContext,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceContext {
    pub include_declaration: bool,
}
