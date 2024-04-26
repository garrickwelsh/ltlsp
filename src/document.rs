use anyhow::Result;
use std::collections::HashMap;

use lsp_types::Diagnostic;

// Manage the documents sent forth by the LSP
pub(crate) trait DocumentManager {
    /// Open a document.
    fn _document_open(&self, url: String, text: String) -> Result<bool>;
}

pub(crate) struct Documents {
    _documents: HashMap<String, Document>,
}
pub(crate) struct Document {
    _uri: String,                  // Needed for the editor to identify feedback
    _diagnostics: Vec<Diagnostic>, // Diagnostics of the file being read
}

impl DocumentManager for Documents {
    fn _document_open(&self, url: String, text: String) -> Result<bool> {
        todo!()
    }
}

impl Documents {
    fn _new() -> Self {
        Self {
            _documents: HashMap::<String, Document>::new(),
        }
    }
}
