// #![allow(clippy::print_stderr)]
#![feature(async_fn_traits)]

use lsp_types::notification::DidOpenTextDocument;
use lsp_types::notification::PublishDiagnostics;
use lsp_types::request::CodeActionRequest;
use lsp_types::request::CodeActionResolveRequest;
use lsp_types::CodeAction;
use lsp_types::CodeActionOrCommand;
use lsp_types::CodeActionResponse;
use lsp_types::Diagnostic;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::{
    request::GotoDefinition, GotoDefinitionResponse, InitializeParams, ServerCapabilities,
};
use lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionProviderCapability, CodeDescription,
    CompletionOptions, OneOf, Position, Range, TextDocumentSyncCapability, TextDocumentSyncKind,
    WorkDoneProgressOptions,
};

use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};

use std::collections::HashMap;
use std::io::Stdout;
use std::{error::Error, fs::OpenOptions};

use tracing::info;

use crate::document::Document;
use crate::lsp_server::Notification;
use crate::tree_sitter::LanguageSitters;

mod config;
mod document;
mod document_checker;
mod languagetool;
mod lsp_server;
mod tree_sitter;

#[cfg(test)]
mod test_utils;

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let _documents = HashMap::<String, Document>::new();
    let mut version: i32 = 0;
    info!("starting example main loop");
    for msg in &connection.receiver {
        info!("got msg: {msg:?}");
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                info!("got request: {req:?}");

                // TODO When get a code need to response with a suggested fix. Code Action - Code below doesn't work.
                match cast::<CodeActionRequest>(req) {
                    Ok((_id, _params)) => {
                        let mut actions: CodeActionResponse = Vec::<CodeActionOrCommand>::new();
                        let action = CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Some title".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: None,
                            edit: None,
                            command: None,
                            is_preferred: None,
                            disabled: None,
                            data: None,
                        });
                        actions.push(action);
                        let result = serde_json::to_value(&actions)?;
                        let resp = Response {
                            id: _id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp.clone()))?;
                        info!("Sent code action response {:?}", resp);
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
            }
            Message::Response(resp) => {
                info!("got response: {resp:?}");
            }
            Message::Notification(not) => {
                info!("got notification: {not:?}");
                if not.method == "textDocument/didOpen" || not.method == "textDocument/didChange" {
                    // Proof of Concept: Send back an error.
                    if let serde_json::Value::Object(map) = not.params {
                        if let serde_json::Value::Object(document_map) = map["textDocument"].clone()
                        {
                            info!("Document text is: {}", document_map["text"]);
                            info!("File uri is: {}", document_map["uri"]);
                        }
                    }

                    // Send a test notification diagnostic.
                    let mut diagnostics = Vec::<Diagnostic>::new();
                    if version == 0 {
                        let diagnostic = Diagnostic {
                            range: Range::new(
                                Position {
                                    line: 1,
                                    character: 1,
                                },
                                Position {
                                    line: 1,
                                    character: 5,
                                },
                            ),
                            // downgrade to hint if we're pointing at the macro
                            severity: Some(lsp_types::DiagnosticSeverity::HINT),
                            code: Some(lsp_types::NumberOrString::String(
                                "An error for as an example".to_string(),
                            )),
                            code_description: None,
                            source: Some("Language tool".to_string()),
                            message: "Fix yee spelling".to_string(),
                            related_information: None,
                            tags: None,
                            data: None, // Some(serde_json::json!({ "rendered": rd.rendered })),
                        };
                        diagnostics.push(diagnostic);
                    }
                    let diagnostic_params = PublishDiagnosticsParams::new(
                        lsp_types::Url::parse("file:///home/gaz/devel/ltlsp/test.ltlsp")?,
                        diagnostics,
                        Some(version),
                    );
                    version += 1;
                    let not = lsp_server::Notification::new(
                        <lsp_types::notification::PublishDiagnostics as lsp_types::notification::Notification>::METHOD
                            .to_owned(),
                        diagnostic_params,
                    );
                    connection.sender.send(Message::Notification(not))?;
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let log_file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("/home/gaz/test.log")?;
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_writer(log_file)
        // .with_writer(std::io::stdout)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    info!("Hello, world!");
    let _config = config::prioritise_config_dirs()?;
    info!("Config dirs: {:?}", _config);
    let _config = config::get_tree_sitter_config()?;
    info!("Config is: {:?}", _config);
    let _config = config::prioritise_runtime_grammar_dirs()?;
    info!("Runtime dirs: {:?}", _config);

    let cfg = config::get_tree_sitter_config()?;
    let _language_sitter = LanguageSitters::new(&cfg.languages)?;

    info!("started language tool server");
    // Start language tool
    let _lt = languagetool::manage_service::LanguageToolRunnerLocal::initialise_language_tool(
        8081, "en-AU",
    )
    .await;

    info!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some([CodeActionKind::QUICKFIX].to_vec()),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
            resolve_provider: None,
        })),
        // TODO: Completion provider needs to filled out to work...
        completion_provider: Some(CompletionOptions {
            resolve_provider: None,
            trigger_characters: None,
            all_commit_characters: None, // Trigger on whitespace?
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
            completion_item: None,
        }),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = match connection.initialize(server_capabilities) {
        Ok(it) => it,
        Err(e) => {
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };
    main_loop(connection, initialization_params)?;
    io_threads.join()?;
    info!("Attempting to drop language tool to shutdown");
    drop(_lt);

    // Shut down gracefully.
    info!("shutting down server");
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
