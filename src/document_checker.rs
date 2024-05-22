use std::collections::HashMap;

use lsp_types::{Diagnostic, Position};
use serde::{Deserialize, Serialize};

use crate::tree_sitter::{LanguageSitterParsers, LanguageSitterResult, LanguageSitters};

pub(crate) trait DocumentLanguageToolCheck<'a> {
    fn parse_str(
        &'a mut self,
        language: &str,
        document_uri: &str,
        document_version: i32,
        document_text: &str,
    ) -> anyhow::Result<Option<&'a DocumentLanguageToolCheckResult>>;
}

pub(crate) struct DocumentLanguageToolChecker {
    language_sitter: LanguageSitters,

    // TODO: Store results for quick actions.
    documents: HashMap<String, DocumentLanguageToolCheckResult>,
}

pub(crate) struct DocumentLanguageToolCheckResult {
    language: String,
    document_uri: String,
    document_version: i32,
    // TODO: Diagnostic with actions.
    diagnostics: Vec<DocumentLanguageToolCheckResultDiagnostic>,
}

pub(crate) struct DocumentLanguageToolCheckResultDiagnostic {
    // TODO: Diagnostic information plus suggested actions/fixes.
    start: Position,
    end: Position,
    code: String,
    code_description: String,
    message: String,
    data: Vec<DocumentLanguageToolCheckResultDiagnosticFix>,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct DocumentLanguageToolCheckResultDiagnosticFix {}

impl DocumentLanguageToolChecker {}

impl<'a> DocumentLanguageToolCheck<'a> for DocumentLanguageToolChecker {
    fn parse_str(
        &'a mut self,
        language: &str,
        document_uri: &str,
        document_version: i32,
        document_text: &str,
    ) -> anyhow::Result<Option<&'a DocumentLanguageToolCheckResult>> {
        if self.documents.contains_key(document_uri)
            && self.documents[document_uri].document_version > document_version
        {
            return Ok(None);
        }
        let x = self.language_sitter.parse_str(language, document_text)?;
        self.documents.insert(
            document_uri.to_string(),
            DocumentLanguageToolCheckResult {
                language: language.to_string(),
                document_uri: document_uri.to_string(),
                document_version,
                diagnostics: x
                    .into_iter()
                    .map(|lsr| lsr.into())
                    .collect::<Vec<DocumentLanguageToolCheckResultDiagnostic>>(),
            },
        );
        Ok(Some(&self.documents[document_uri]))
    }
}

impl From<LanguageSitterResult> for DocumentLanguageToolCheckResultDiagnostic {
    fn from(value: LanguageSitterResult) -> Self {
        Self {
            start: lsp_types::Position::new(value.start_row, value.end_row),
            end: todo!(),
            code: todo!(),
            code_description: todo!(),
            message: todo!(),
            data: todo!(),
        }
    }
}

impl From<DocumentLanguageToolCheckResultDiagnostic> for Diagnostic {
    fn from(value: DocumentLanguageToolCheckResultDiagnostic) -> Self {
        Self {
            range: lsp_types::Range::new(value.start, value.end),
            severity: Some(lsp_types::DiagnosticSeverity::HINT),
            code: Some(lsp_types::NumberOrString::String(value.code.clone())),
            code_description: None,
            source: Some("Language tool".to_string()),
            message: value.code_description.clone(),
            related_information: None,
            tags: None,
            data: None, // TODO: Send data to lsp_client
        }
    }
}
