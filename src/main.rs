// #![allow(clippy::print_stderr)]
#![feature(async_fn_traits)]

use anyhow::Context;
use lsp_server::ResponseError;
use lsp_types::request::CodeActionRequest;
use lsp_types::CodeAction;
use lsp_types::CodeActionOrCommand;
use lsp_types::CodeActionResponse;
use lsp_types::Diagnostic;
use lsp_types::DocumentChanges;
use lsp_types::OptionalVersionedTextDocumentIdentifier;
use lsp_types::PublishDiagnosticsParams;
use lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionProviderCapability, OneOf, Range,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};
use lsp_types::{InitializeParams, ServerCapabilities};

use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};

use std::collections::HashMap;
use std::{error::Error, fs::OpenOptions};

use tracing::info;

pub(crate) use crate::document::Document;
use crate::document_checker::DocumentLanguageToolCheck;
use crate::document_checker::DocumentLanguageToolChecker;
use crate::tree_sitter::LanguageSitters;

mod config;
mod document;
mod document_checker;
mod languagetool;
mod lsp_server;
mod tree_sitter;

#[cfg(test)]
mod test_utils;

async fn main_loop(connection: Connection, params: serde_json::Value) -> anyhow::Result<()> {
    // Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    let _documents = HashMap::<String, Document>::new();
    let mut document_checker = DocumentLanguageToolChecker::new().await;
    info!("starting example main loop");
    for msg in &connection.receiver {
        info!("got msg: {msg:?}");
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                info!("got request: {req:?}");

                let serde_json::Value::Object(map) = req.params.clone() else {
                    return anyhow::Result::Err(anyhow::anyhow!("Unable to find map"));
                };
                // TODO When get a code need to response with a suggested fix. Code Action - Code below doesn't work.
                match cast::<CodeActionRequest>(req) {
                    Ok((_id, code_action_params)) => {
                        info!("{map:?}");
                        let diagnostics = map
                            .get("context")
                            .context("Context for code action doesn't exist as expected. ")?
                            .as_object()
                            .context("context was not an object")?
                            .get("diagnostics")
                            .context("diagnostics were unable to be found for code actions")?
                            .as_array()
                            .context("diagnostics were not an array")?;
                        for i in diagnostics {
                            let Some(diagnostic) = i
                                .as_object()
                                .and_then(|j| j.get("data"))
                                .and_then(|j| j.as_i64())
                                .and_then(|id| {
                                    document_checker.get_diagnostic(
                                        code_action_params.text_document.uri.as_str(),
                                        id,
                                    )
                                })
                            else {
                                info!("Did not much diagnostic ignore.");
                                let resp = Response {
                                    id: _id.clone(),
                                    result: Some(serde_json::to_value(
                                        &Vec::<CodeActionOrCommand>::new(),
                                    )?),
                                    error: None, //Some(ResponseError { code: lsp_types::error_codes::REQUEST_FAILED as i32, message: "Request does not appear to have been meant for this server".to_string(), data: None })
                                };
                                connection.sender.send(Message::Response(resp.clone()))?;
                                continue;
                            };

                            let code_actions = &diagnostic.code_actions;
                            info!("Code actions are: {:?}", code_actions);
                            let mut actions: CodeActionResponse = Vec::<CodeActionOrCommand>::new();
                            for code_action in code_actions {
                                let action = CodeActionOrCommand::CodeAction(CodeAction {
                                    title: code_action.value.to_string(),
                                    kind: Some(CodeActionKind::QUICKFIX),
                                    diagnostics: None,
                                    edit: Some(lsp_types::WorkspaceEdit {
                                        changes: None,
                                        change_annotations: None,
                                        document_changes: Some(DocumentChanges::Edits(
                                            [lsp_types::TextDocumentEdit {
                                                text_document:
                                                    OptionalVersionedTextDocumentIdentifier {
                                                        uri: code_action_params
                                                            .text_document
                                                            .uri
                                                            .clone(),
                                                        version: None,
                                                    },
                                                edits: [OneOf::Left(lsp_types::TextEdit {
                                                    range: Range {
                                                        start: diagnostic.start,
                                                        end: diagnostic.end,
                                                    },
                                                    new_text: code_action.value.to_string(),
                                                })]
                                                .to_vec(),
                                            }]
                                            .to_vec(),
                                        )),
                                    }),
                                    command: None,
                                    is_preferred: None,
                                    disabled: None,
                                    data: None,
                                });
                                actions.push(action);
                            }
                            let result = serde_json::to_value(&actions)?;
                            let resp = Response {
                                id: _id.clone(),
                                result: Some(result),
                                error: None,
                            };
                            connection.sender.send(Message::Response(resp.clone()))?;
                            info!("Sent code action response {:?}", resp);
                        }
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
                    if let serde_json::Value::Object(map) = not.params {
                        if let serde_json::Value::Object(document_map) = map["textDocument"].clone()
                        {
                            if not.method == "textDocument/didOpen" {
                                info!("Document opened");
                                process_document(
                                    &connection,
                                    &mut document_checker,
                                    &document_map,
                                    document_map["text"]
                                        .as_str()
                                        .context("Expected document uri")?,
                                )
                                .await?;
                            } else if not.method == "textDocument/didChange" {
                                info!("Document changed {map:?}");
                                if let serde_json::Value::Array(change_map) =
                                    map["contentChanges"].clone()
                                {
                                    info!("process content changes, change_map {change_map:?}");
                                    process_document(
                                        &connection,
                                        &mut document_checker,
                                        &document_map,
                                        change_map[0]
                                            .as_object()
                                            .context("Expected document uri")?["text"]
                                            .as_str()
                                            .context("Expected document text")?,
                                    )
                                    .await?;
                                } else {
                                    info!("No content changes");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

async fn process_document(
    connection: &Connection,
    document_checker: &mut DocumentLanguageToolChecker,
    document_map: &serde_json::Map<String, serde_json::Value>,
    document_text: &str,
) -> anyhow::Result<()> {
    let mut diagnostics = Vec::<Diagnostic>::new();
    info!("Running document checker.");
    let uri = document_map["uri"]
        .as_str()
        .context("Expected document uri")?;
    let language = match document_checker.get_language(uri) {
        Some(res) => res,
        None => document_map["languageId"]
            .as_str()
            .context("Expected a language")?,
    }
    .to_string();
    let document_parsed = document_checker
        .parse_str(
            &language,
            uri,
            i32::try_from(
                document_map["version"]
                    .as_i64()
                    .context("Expected a version")?,
            )?,
            document_text,
        )
        .await?;
    // If None then a newer version has already been received. So do nothing (do not process).
    let Some(document_parsed) = document_parsed else {
        return Ok(());
    };

    for i in &document_parsed.diagnostics {
        diagnostics.push(i.into());
    }
    let diagnostic_params = PublishDiagnosticsParams::new(
        lsp_types::Url::parse(&document_parsed.document_uri)?,
        diagnostics,
        Some(document_parsed.document_version),
    );
    let not = lsp_server::Notification::new(
        <lsp_types::notification::PublishDiagnostics as lsp_types::notification::Notification>::METHOD
            .to_owned(),
        diagnostic_params,
        );
    connection.sender.send(Message::Notification(not))?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let result = main_inner().await;
    info!("{:?}", result);
    result
}
async fn main_inner() -> Result<(), Box<dyn Error + Sync + Send>> {
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
            resolve_provider: Some(false),
        })),
        // TODO: Completion provider needs to filled out to work...
        completion_provider: None,
        // completion_provider: Some(CompletionOptions {
        //     resolve_provider: None,
        //     trigger_characters: None,
        //     all_commit_characters: None, // Trigger on whitespace?
        //     work_done_progress_options: WorkDoneProgressOptions {
        //         work_done_progress: None,
        //     },
        //     completion_item: None,
        // }),
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
    main_loop(connection, initialization_params).await?;
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
