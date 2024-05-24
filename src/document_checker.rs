use std::{collections::HashMap, ops::Range};

use crate::languagetool::{LanguageToolRequestBuilder, LanguageToolResultMatch};
use lsp_types::{Diagnostic, Position};
use serde::{Deserialize, Serialize};

use crate::{
    languagetool::{
        manage_service::{LanguageToolInitialisation, LanguageToolRunner, LanguageToolRunnerLocal},
        LanguageToolResult,
    },
    tree_sitter::{LanguageSitterParsers, LanguageSitterResult, LanguageSitters},
};

pub(crate) trait DocumentLanguageToolCheck {
    async fn parse_str(
        &mut self,
        language: &str,
        document_uri: &str,
        document_version: i32,
        document_text: &str,
    ) -> anyhow::Result<Option<&DocumentLanguageToolCheckResult>>;
}

pub(crate) struct DocumentLanguageToolChecker {
    language_sitter: LanguageSitters,
    language_tool: LanguageToolRunnerLocal,

    // TODO: Store results for quick actions.
    documents: HashMap<String, DocumentLanguageToolCheckResult>,
}

pub(crate) struct DocumentLanguageToolCheckResult {
    language: String,
    document_uri: String,
    document_version: i32,
    diagnostics: Vec<DocumentLanguageToolCheckChunkResult>,
}

pub(crate) struct DocumentLanguageToolCheckChunkResult {
    start: Position,
    end: Position,
    code: String,
    diagnostics: Vec<LanguageToolResult>,
}

impl DocumentLanguageToolChecker {}

impl DocumentLanguageToolCheck for DocumentLanguageToolChecker {
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
        let chunks = self.language_sitter.parse_str(language, document_text)?;
        let dt_bytes = document_text.as_bytes();
        let mut results = Vec::<LanguageToolResult>::new();
        let mut request = self.language_tool.new_request();
        let mut lastoffset: i32 = 0;
        for chunk in chunks {
            if chunk.start_pos > lastoffset + 1 {
                request.add_markup(std::str::from_utf8(
                    dt_bytes
                        .get(Range::<usize> {
                            start: i32::try_into(lastoffset)?,
                            end: i32::try_into(chunk.start_pos - 1)?,
                        })
                        .expect("Unable to get value"),
                )?);
            }
            request.add_text(std::str::from_utf8(
                dt_bytes
                    .get(Range::<usize> {
                        start: i32::try_into(chunk.start_pos)?,
                        end: i32::try_into(chunk.end_pos)?,
                    })
                    .expect("Unable to get value"),
            )?);
            results.push(request.execute_request().await?);
        }
        // self.documents.insert(
        //     document_uri.to_string(),
        //     DocumentLanguageToolCheckResult {
        //         language: language.to_string(),
        //         document_uri: document_uri.to_string(),
        //         document_version,
        //         diagnostics: results
        //             .into_iter()
        //             .map(|ltr| Into::<Vec<Diagnostic>>::into(ltr))
        //             .flatten()
        //             .collect(),
        //     },
        // );
        Ok(Some(&self.documents[document_uri]))
    }
}

impl From<LanguageToolResult> for Vec<DocumentLanguageToolCheckChunkResult> {
    fn from(value: LanguageToolResult) -> Self {
        let mut retval = Vec::<DocumentLanguageToolCheckChunkResult>::new();
        for diagnostic in value.matches {
            retval.push(DocumentLanguageToolCheckChunkResult {
                start: todo!(),
                end: todo!(),
                code: todo!(),
                diagnostics: todo!(),
            });
        }
        retval
    }
}
