use std::{fmt, io};

use headers::Headers;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, ErrorKind};

pub mod diagnostics;
pub mod document_symbol;
pub mod hover;
pub mod initialize;
pub mod reference;
pub mod text;

mod headers;

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Request(Request),
    Response(Response),
    Notification(Notification),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: RequestId,
    pub method: String,

    #[serde(default = "serde_json::Value::default")]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: RequestId,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseError {
    pub code: i32,
    pub message: String,
}

/// Error codes coppied over from the
/// [specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#errorCodes)
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerErrorStart = -32099,
    ServerErrorEnd = -32000,
    ServerNotInitialized = -32002,
    UnknownErrorCode = -32001,
    RequestCanceled = -32800,
    ContentModified = -32801,
    ServerCancelled = -32802,
    RequestFailed = -32803,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub method: String,

    #[serde(default = "serde_json::Value::default")]
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Int(i64),
}

impl From<i64> for RequestId {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<String> for RequestId {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestId::String(id) => fmt::Display::fmt(id, f),
            // Use Debug instead of Display to differentiate
            // between 42 and "42".
            RequestId::Int(id) => fmt::Debug::fmt(id, f),
        }
    }
}

impl From<Request> for Message {
    fn from(value: Request) -> Self {
        Self::Request(value)
    }
}

impl From<Response> for Message {
    fn from(value: Response) -> Self {
        Self::Response(value)
    }
}

impl From<Notification> for Message {
    fn from(value: Notification) -> Self {
        Self::Notification(value)
    }
}

impl Message {
    pub fn read<R: io::Read + io::BufRead>(input: &mut R) -> io::Result<Self> {
        let headers = Headers::read(input)?;

        let mut buf = vec![0_u8; headers.content_length];
        input.read_exact(&mut buf)?;

        let msg = serde_json::from_slice(&buf)?;
        Ok(msg)
    }

    pub fn write(self, output: &mut impl io::Write) -> io::Result<()> {
        #[derive(Serialize)]
        struct JsonRpc {
            jsonrpc: &'static str,
            #[serde(flatten)]
            message: Message,
        }

        let data = serde_json::to_string(&JsonRpc {
            jsonrpc: "2.0",
            message: self,
        })?;

        let headers = Headers::with_length(data.len());

        write!(output, "{}", headers)?;
        write!(output, "{}", data)?;
        output.flush()
    }
}

impl From<Error> for Response {
    fn from(value: Error) -> Self {
        let err = match value.kind {
            ErrorKind::ExtractError(err) => ResponseError {
                code: ErrorCode::InvalidParams as i32,
                message: format!("Invalid parameters: {}", err),
            },
        };

        Self {
            id: value.request_id,
            result: None,
            error: Some(err),
        }
    }
}

impl Response {
    pub fn new_ok<R: Serialize>(id: RequestId, result: R) -> Response {
        Response {
            id,
            result: Some(serde_json::to_value(result).unwrap()),
            error: None,
        }
    }

    pub fn new_err(id: RequestId, code: i32, message: String) -> Response {
        let error = ResponseError { code, message };

        Response {
            id,
            result: None,
            error: Some(error),
        }
    }
}

impl Request {
    pub fn extract<P: DeserializeOwned>(self) -> Result<(RequestId, P), Error> {
        let params = serde_json::from_value(self.params).map_err(|err| Error {
            request_id: self.id.clone(),
            kind: ErrorKind::ExtractError(err),
        })?;

        Ok((self.id, params))
    }
}

impl Notification {
    pub fn new<P: Serialize>(method: String, params: P) -> Notification {
        Notification {
            method,
            params: serde_json::to_value(params).unwrap(),
        }
    }

    pub fn extract<P: DeserializeOwned>(self) -> Result<P, ErrorKind> {
        let params = serde_json::from_value(self.params).map_err(ErrorKind::ExtractError)?;

        Ok(params)
    }
}
