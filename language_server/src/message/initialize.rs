use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientServerInfo {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeClientParams {
    pub client_info: ClientServerInfo,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub server_info: ClientServerInfo,
    pub capabilities: ServerCapabilities,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    pub text_document_sync: TextDocumentSyncOptions,
    pub definition_provider: bool,
    pub document_highlight_provider: bool,
    pub references_provider: bool,
    pub hover_provider: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentSyncOptions {
    pub open_close: bool,
    pub change: u8,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum TextDocumentSyncKind {
    None = 0,
    Full = 1,
    Incremental = 2,
}
