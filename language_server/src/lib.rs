use core::fmt;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use analyze::{analyze, document_info::DocumentInfo};
use error::{Error, ErrorKind};
use message::{initialize::*, *};
use parser::position::PositionOrdering;
use reference::ReferenceParams;
use text::*;

pub mod error;

mod analyze;
mod message;

#[derive(Clone, Copy)]
#[allow(dead_code)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub struct Server {
    log_file: Option<fs::File>,
    running: bool,
    documents: HashMap<String, DocumentInfo>,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl Server {
    pub fn new(log_file_path: Option<PathBuf>) -> Self {
        let log_file = log_file_path.map(|path| {
            // It's fine to unwrap, since we want to
            // exit if file doesn't exist.
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .expect("Failed to open log file")
        });

        Self {
            log_file,
            running: false,
            documents: HashMap::new(),
        }
    }

    fn log(&mut self, level: LogLevel, message: &str) {
        let Some(file) = &mut self.log_file else {
            return;
        };

        writeln!(file, "[{}]: {}", level, message).expect("Failed to write to log file");
        file.flush().expect("Failed to write to log file");
    }

    pub fn start(&mut self) {
        let mut stdin = io::stdin().lock();
        let mut stdout = io::stdout().lock();

        self.log(LogLevel::Info, "Starting the server");

        self.running = true;
        while self.running {
            // It's fine to unwrap. If stdin/stdout pipe is broken, we can't
            // do anything else but fail.
            let message = Message::read(&mut stdin).unwrap();

            match message {
                Message::Request(req) => {
                    let resp: Response = match self.handle_request(req) {
                        Ok(resp) => resp,
                        Err(err) => {
                            self.log(LogLevel::Error, &format!("{}", err));
                            err.into()
                        }
                    };

                    let msg: Message = resp.into();
                    msg.write(&mut stdout).unwrap();
                    self.log(LogLevel::Debug, "Writtten response");
                }
                Message::Notification(notification) => {
                    let res = self.handle_notification(notification);
                    if let Err(err) = res {
                        self.log(
                            LogLevel::Error,
                            &format!("Invalid notification params: {}", err),
                        );
                    }
                }
                Message::Response(_) => {
                    unreachable!("Got response from client instead of request/notification")
                }
            }
        }

        self.log(LogLevel::Info, "Exiting the server");
    }

    fn handle_notification(&mut self, notification: Notification) -> Result<(), ErrorKind> {
        self.log(
            LogLevel::Debug,
            &format!("Got notification: {}", &notification.method),
        );

        match notification.method.as_ref() {
            "textDocument/didOpen" => {
                let params: DidOpenTextDocumentParams = notification.extract()?;
                let params = params.text_document;
                self.log(
                    LogLevel::Info,
                    &format!("Setting contents for opened file: {}", params.uri),
                );

                self.set_document_info(params.uri, params.text);
            }
            "textDocument/didChange" => {
                let mut params: DidChangeTextDocumentParams = notification.extract()?;
                self.log(
                    LogLevel::Info,
                    &format!("Updating contents for file: {}", params.text_document.uri),
                );

                if let Some(content) = params.content_changes.pop() {
                    self.set_document_info(params.text_document.uri, content.text)
                }
            }
            "textDocument/didClose" => {
                let params: DidCloseTextDocumentParams = notification.extract()?;
                self.log(
                    LogLevel::Info,
                    &format!("Closing file: {}", params.text_document.uri),
                );

                self.documents.remove(&params.text_document.uri);
            }
            "exit" => {
                self.running = false;
            }
            _ => (),
        }

        Ok(())
    }

    fn handle_request(&mut self, req: Request) -> Result<Response, Error> {
        self.log(
            LogLevel::Info,
            &format!("Got request, id: {}, method: {}", req.id, req.method),
        );

        let resp = match req.method.as_ref() {
            "initialize" => Response::new_ok(req.id, self.get_capabilities()),
            "shutdown" => Response::new_ok(req.id, serde_json::Value::Null),
            "textDocument/definition" => {
                let (req_id, params) = req.extract::<TextDocumentPositionParams>()?;

                let doc_info = self.documents.get(&params.text_document.uri);
                let mut res: Option<Location> = None;
                if let Some(doc_info) = doc_info {
                    res = doc_info
                        .get_definition(&params.position)
                        .map(|def| Location::new(params.text_document.uri.to_string(), def));
                }

                Response::new_ok(req_id, res)
            }
            "textDocument/documentHighlight" => {
                let (req_id, params) = req.extract::<TextDocumentPositionParams>()?;

                let doc_info = self.documents.get(&params.text_document.uri);
                let mut res: Option<Vec<DocumentHighlight>> = None;
                if let Some(doc_info) = doc_info {
                    res = doc_info.get_references(&params.position).map(|ranges| {
                        ranges
                            .iter()
                            .map(|rng| DocumentHighlight { range: *rng })
                            .collect()
                    });
                }

                Response::new_ok(req_id, res)
            }
            "textDocument/references" => {
                let (req_id, params) = req.extract::<ReferenceParams>()?;
                let doc_name = params.text_position.text_document.uri.clone();
                let pos = params.text_position.position;

                let doc_info = self.documents.get(&doc_name);
                let mut res: Option<Vec<Location>> = None;
                if let Some(doc_info) = doc_info {
                    res = doc_info.get_references(&pos).map(|ranges| {
                        ranges
                            .iter()
                            .filter_map(|rng| {
                                if params.context.include_declaration
                                    || pos.cmp_range(rng) != PositionOrdering::Inside
                                {
                                    Some(Location::new(doc_name.clone(), *rng))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    });
                }

                Response::new_ok(req_id, res)
            }
            method => {
                self.log(LogLevel::Warn, &format!("Got unknown method: {method}"));
                Response::new_err(
                    req.id,
                    ErrorCode::MethodNotFound as i32,
                    "Unknown method".to_string(),
                )
            }
        };

        Ok(resp)
    }

    fn get_capabilities(&self) -> InitializeResult {
        InitializeResult {
            server_info: ClientServerInfo {
                name: "AOC LSP".to_string(),
                version: None,
            },
            capabilities: ServerCapabilities {
                text_document_sync: TextDocumentSyncOptions {
                    open_close: true,
                    change: TextDocumentSyncKind::Full as u8,
                },
                definition_provider: true,
                document_highlight_provider: true,
                references_provider: true,
            },
        }
    }

    fn set_document_info(&mut self, name: String, content: String) {
        let Ok(program) = parser::parse(&content) else {
            self.log(LogLevel::Warn, "failed to parse document");
            self.documents.insert(name, DocumentInfo::default());
            return;
        };

        let document_info = analyze(&program);
        self.documents.insert(name, document_info);
    }
}
