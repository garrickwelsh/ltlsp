use std::{
    collections::HashMap,
    ops::Range,
    sync::atomic::{AtomicI32, AtomicI64},
};

use crate::languagetool::{LanguageToolRequestBuilder, LanguageToolResultMatch};
use anyhow::Context;
use lsp_types::{CodeAction, Diagnostic, DiagnosticSeverity, Position};
use serde::{Deserializer, Serialize};
use tracing::{debug, info};

use crate::{
    languagetool::manage_service::{LanguageToolRunner, LanguageToolRunnerLocal},
    tree_sitter::{LanguageSitterParsers, LanguageSitters},
};

pub(crate) trait DocumentLanguageToolCheck {
    async fn parse_str(
        &mut self,
        language: &str,
        document_uri: &str,
        document_version: i32,
        document_text: &str,
    ) -> anyhow::Result<Option<&DocumentLanguageToolCheckResult>>;

    fn get_language(&self, document_uri: &str) -> Option<&str>;

    fn get_diagnostic(
        &self,
        document_uri: &str,
        id: i64,
    ) -> Option<&DocumentLanguageToolCheckChunkResult>;
}

#[derive(Debug)]
pub(crate) struct DocumentLanguageToolChecker {
    language_sitter: LanguageSitters,
    language_tool: LanguageToolRunnerLocal,

    // TODO: Store results for quick actions.
    documents: HashMap<String, DocumentLanguageToolCheckResult>,
}

#[derive(Debug)]
pub(crate) struct DocumentLanguageToolCheckResult {
    pub(crate) language: String,
    pub(crate) document_uri: String,
    pub(crate) document_version: i32,
    pub(crate) diagnostics: Vec<DocumentLanguageToolCheckChunkResult>,
}

#[derive(Debug)]
pub(crate) struct DocumentLanguageToolCheckChunkResult {
    pub(crate) id: i64,
    pub(crate) start: Position,
    pub(crate) end: Position,
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) short_message: String,
    pub(crate) code_actions: Vec<DocumentLanguageToolCheckChunkResultCodeAction>,
}

#[derive(Debug)]
pub(crate) struct DocumentLanguageToolCheckChunkResultCodeAction {
    pub(crate) value: String,
}

impl DocumentLanguageToolChecker {
    // #[tracing::instrument]
    pub(crate) async fn new() -> Self {
        let cfg = crate::config::get_tree_sitter_config().unwrap();
        let language_sitter = LanguageSitters::new(&cfg.languages).unwrap();
        let language_tool =
            LanguageToolRunnerLocal::initialise_language_tool_local(8081, "en-AU").await;
        Self {
            language_sitter,
            language_tool,
            documents: HashMap::<String, DocumentLanguageToolCheckResult>::new(),
        }
    }
}

impl DocumentLanguageToolCheck for DocumentLanguageToolChecker {
    // #[tracing::instrument]
    async fn parse_str(
        &mut self,
        language: &str,
        document_uri: &str,
        document_version: i32,
        document_text: &str,
    ) -> anyhow::Result<Option<&DocumentLanguageToolCheckResult>> {
        if self.documents.contains_key(document_uri)
            && self.documents[document_uri].document_version > document_version
        {
            return Ok(None);
        }
        info!("Start document checker");
        self.language_sitter.initialise(language)?;
        let chunks = self.language_sitter.parse_str(language, document_text)?;
        info!("Parse the document");
        info!("Text parsed found {} comments", chunks.len());
        let dt_bytes = document_text.as_bytes();
        let mut request = self.language_tool.new_request();
        let mut lastoffset: i32 = 0;
        // info!("document_text: '{:?}", document_text);
        for chunk in chunks {
            // info!("chunk: '{:?}", chunk);
            if chunk.start_pos > lastoffset {
                let mark_up = std::str::from_utf8(
                    dt_bytes
                        .get(Range::<usize> {
                            start: i32::try_into(lastoffset)?,
                            end: i32::try_into(chunk.start_pos)?,
                        })
                        .expect("Unable to get value"),
                )?;
                // info!("mark_up: {}", mark_up);
                request.add_markup(mark_up);
            }
            let text = std::str::from_utf8(
                dt_bytes
                    .get(Range::<usize> {
                        start: i32::try_into(chunk.start_pos)?,
                        end: i32::try_into(chunk.end_pos)?,
                    })
                    .expect("Unable to get value"),
            )?;
            // info!("text: {}", text);
            request.add_text(text);
            lastoffset = chunk.end_pos;
        }
        let length: i32 = i32::try_from(dt_bytes.len())?;
        if lastoffset < length - 1 {
            let mark_up = std::str::from_utf8(
                dt_bytes
                    .get(Range::<usize> {
                        start: i32::try_into(lastoffset + 1)?,
                        end: usize::try_into(dt_bytes.len())?,
                    })
                    .expect("Unable to get value"),
            )?;
            // info!("mark_up: {}", mark_up);
            request.add_markup(mark_up);
        }
        let result = request.execute_request().await?;
        self.documents.insert(
            document_uri.to_string(),
            DocumentLanguageToolCheckResult {
                language: language.to_string(),
                document_uri: document_uri.to_string(),
                document_version,
                diagnostics: result
                    .matches
                    .into_iter()
                    .map(|ltr| ltr.into_language_tool_result(document_text).unwrap())
                    .collect(),
            },
        );
        Ok(Some(&self.documents[document_uri]))
    }

    fn get_language(&self, document_uri: &str) -> Option<&str> {
        let doc = self.documents.get(document_uri)?;
        Some(&doc.language)
    }

    fn get_diagnostic(
        &self,
        document_uri: &str,
        id: i64,
    ) -> Option<&DocumentLanguageToolCheckChunkResult> {
        info!("Get diagnostic");
        let document = self.documents.get(document_uri)?;
        let diagnostic = document.diagnostics.iter().filter(|d| d.id == id).last()?;
        Some(diagnostic)
    }
}

impl LanguageToolResultMatch {
    // #[tracing::instrument]
    fn into_language_tool_result(
        &self,
        document_text: &str,
    ) -> anyhow::Result<DocumentLanguageToolCheckChunkResult> {
        static ID_COUNTER: AtomicI64 = AtomicI64::new(1);
        let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Release);

        debug!("{:?}", self);
        let li = line_index::LineIndex::new(document_text);
        let start = li.line_col(line_index::TextSize::new(self.offset as u32));
        let end = li.line_col(line_index::TextSize::new(
            (self.offset + self.length) as u32,
        ));
        Ok(DocumentLanguageToolCheckChunkResult {
            id,
            start: Position {
                line: start.line,
                character: start.col,
            },
            end: Position {
                line: end.line,
                character: end.col,
            },
            code: self.rule.id.to_string(),
            message: self.message.to_string(),
            short_message: self.short_message.to_string(),
            code_actions: self.replacements.iter().map(|r| r.into()).collect(),
        })
    }
}

impl From<&crate::languagetool::LanguageToolResultListItem>
    for DocumentLanguageToolCheckChunkResultCodeAction
{
    fn from(value: &crate::languagetool::LanguageToolResultListItem) -> Self {
        Self {
            value: value.value.to_string(),
        }
    }
}

impl From<&DocumentLanguageToolCheckChunkResultCodeAction> for CodeAction {
    fn from(value: &DocumentLanguageToolCheckChunkResultCodeAction) -> Self {
        Self {
            title: format!("Replace with \"{}\"", value.value),
            kind: Some(lsp_types::CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: None, // TODO Some stuff to do...
            command: None,
            is_preferred: None,
            disabled: None,
            data: None,
        }
    }
}

impl From<&DocumentLanguageToolCheckChunkResult> for Diagnostic {
    fn from(value: &DocumentLanguageToolCheckChunkResult) -> Self {
        Self {
            range: lsp_types::Range::new(value.start, value.end),
            severity: Some(DiagnosticSeverity::INFORMATION),
            code: Some(lsp_types::NumberOrString::String(value.code.to_string())),
            code_description: None,
            source: Some("Ltlsp: LanguageTool".to_string()),
            message: value.message.to_string(),
            related_information: None,
            tags: None,
            data: Some(serde_json::Value::Number(value.id.into())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::DocumentLanguageToolChecker;
    use super::*;

    #[tokio::test]
    async fn test_verify_rust_parsing_and_language_tool() -> anyhow::Result<()> {
        let _ = crate::test_utils::setup_tracing();
        let rust = r###"
 // This is a comment, and is ignored by the compiler.
// You can test this code by clicking the "Run" button over there ->
// or if you prefer to use your keyboard, you can use the "Ctrl + Enter"
// shortcut.

// This code is editable, feel free to hack it!
// You can always return to the original code by clicking the "Reset" button ->

//! This is the main function.
/* Another comment type */
fn main() {file:///home/gaz/devel/ltlsp/test.ltlsp
    // Statements here are executed when the compiled binary is called.

    // Print text to the console.
    println!("Hello World!");
}
"###;
        let mut document_checker = DocumentLanguageToolChecker::new().await;
        let _result = document_checker.parse_str("rust", "", 0, rust).await;
        info!("Parse rust has been actioned");
        Ok(())
    }
}
