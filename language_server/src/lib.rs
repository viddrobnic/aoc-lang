use core::fmt;
use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::PathBuf,
};

use error::Error;
use message::{initialize::*, *};
use text::{DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams};

pub mod error;
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
    documents: HashMap<String, String>,
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

        loop {
            // It's fine to unwrap. If stdin/stdout pipe is broken, we can't
            // do anything else but fail.
            let message = Message::read(&mut stdin).unwrap();

            match message {
                Message::Request(req) => {
                    let resp = self.handle_request(req);
                    let msg: Message = resp.into();
                    msg.write(&mut stdout).unwrap();
                    self.log(LogLevel::Debug, "Writtten response")
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
    }

    fn handle_notification(&mut self, notification: Notification) -> Result<(), Error> {
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

                self.documents.insert(params.uri, params.text);
            }
            "textDocument/didChange" => {
                let mut params: DidChangeTextDocumentParams = notification.extract()?;
                self.log(
                    LogLevel::Info,
                    &format!("Updating contents for file: {}", params.text_document.uri),
                );

                if let Some(content) = params.content_changes.pop() {
                    self.documents
                        .insert(params.text_document.uri, content.text);
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
            _ => (),
        }

        Ok(())
    }

    fn handle_request(&mut self, req: Request) -> Response {
        self.log(
            LogLevel::Info,
            &format!("Got request, id: {}, method: {}", req.id, req.method),
        );

        match req.method.as_ref() {
            "initialize" => Response::new_ok(req.id, self.get_capabilities()),
            method => {
                self.log(LogLevel::Warn, &format!("Got unknown method: {method}"));
                Response::new_err(
                    req.id,
                    ErrorCode::MethodNotFound as i32,
                    "Unknown method".to_string(),
                )
            }
        }
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
                diagnostic_provider: DiagnosticOptions {
                    inter_file_dependencies: false,
                    workspace_diagnostics: false,
                },
            },
        }
    }
}
