use std::collections::HashMap;
use std::path::PathBuf;

use ::tree_sitter::Node;
use ::tree_sitter::TextProvider;
use ::tree_sitter::Tree;
use ::tree_sitter::{Language, Parser};
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use tracing::info;

use crate::config::*;
use crate::tree_sitter;

#[cfg(unix)]
const DYLIB_EXTENSION: &str = "so";

#[cfg(windows)]
const DYLIB_EXTENSION: &str = "dll";

#[cfg(target_arch = "wasm32")]
const DYLIB_EXTENSION: &str = "wasm";

pub(crate) fn get_language(
    language_name: &str,
    language_library_name: &str,
    language_library_search_path: &Vec<PathBuf>,
) -> Result<Language> {
    use std::path::PathBuf;

    info!("Do something {:?}", language_library_search_path);
    use libloading::{Library, Symbol};
    let mut rel_library_path = PathBuf::new().join(language_library_name);
    rel_library_path.set_extension(DYLIB_EXTENSION);

    info!("{:?}", rel_library_path);
    for path_buf in language_library_search_path {
        let lib_path_buf = path_buf.join(&rel_library_path);
        info!("{:?}", lib_path_buf);
        if lib_path_buf.exists() {
            let library = unsafe { Library::new(&lib_path_buf) }
                .with_context(|| format!("Error opening dynamic library {:?}", lib_path_buf))?;
            let language_fn_name = format!("tree_sitter_{}", language_name.replace('-', "_"));
            let language = unsafe {
                let language_fn: Symbol<unsafe extern "C" fn() -> Language> = library
                    .get(language_fn_name.as_bytes())
                    .with_context(|| format!("Failed to load symbol {}", language_fn_name))?;
                language_fn()
            };
            std::mem::forget(library);
            return Ok(language);
        }
    }
    Result::Err(anyhow::anyhow!(
        "Unable to find tree sitter library {}",
        language_library_name
    ))
}

pub(crate) trait LanguageSitterParsers {
    fn is_initialised(&self, language: &str) -> bool;
    fn initialise(&mut self, language: &str) -> Result<()>;
    fn parse_str<'a>(&self, language: &'a str, s: &'a str) -> Result<Vec<LanguageSitterResult>>;
}

pub(crate) trait LanguageSitterParser {
    fn parse_str<'a>(&self, s: &'a str) -> Result<Vec<LanguageSitterResult>>;
}

#[derive(Debug)]
pub(crate) struct LanguageSitterResult {
    text: String,
    start_pos: usize,
    end_pos: usize,
}
#[derive(Debug)]
pub(crate) struct LanguageSitters {
    language_parsers_uninitialised: HashMap<String, LanguageSitterUninitialised>,
    language_parsers_initialised: HashMap<String, LanguageSitterInitialised>,
}

#[derive(Debug)]
pub(crate) struct LanguageSitterUninitialised {
    language_name: String,
    language_library_name: String,
    language_library_search_path: Vec<PathBuf>,
    nodes_to_check: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct LanguageSitterInitialised {
    language_name: String,
    language_library_name: String,
    language_library_search_path: Vec<PathBuf>,
    nodes_to_check: Vec<String>,
    language: Language,
    nodes_to_query: Vec<::tree_sitter::Query>,
}

impl LanguageSitters {
    pub(crate) fn new(
        language_configs: &HashMap<String, LanguageSitterConfigNode>,
    ) -> Result<LanguageSitters> {
        let mut uninitalised = HashMap::<String, LanguageSitterUninitialised>::new();
        for (language, language_config) in language_configs {
            uninitalised.insert(
                language.clone(),
                LanguageSitterUninitialised::new(language, language_config)?,
            );
        }
        Ok(Self {
            language_parsers_uninitialised: uninitalised,
            language_parsers_initialised: HashMap::<String, LanguageSitterInitialised>::new(),
        })
    }
}

impl LanguageSitterUninitialised {
    pub(crate) fn new(language: &str, language_config: &LanguageSitterConfigNode) -> Result<Self> {
        Ok(Self {
            language_name: language.to_string(),
            language_library_name: language_config.language_library_name.clone(),
            nodes_to_check: language_config.expressions.clone(),
            language_library_search_path: crate::config::prioritise_runtime_grammar_dirs()?,
        })
    }

    pub(crate) fn initialise(&self) -> Result<LanguageSitterInitialised> {
        info!("LanguageSitterUninitialised: {:?}", self);
        let language = get_language(
            &self.language_name,
            &self.language_library_name,
            &self.language_library_search_path,
        )?;
        let mut language_sitter = LanguageSitterInitialised {
            language_name: self.language_name.clone(),
            language_library_name: self.language_library_name.clone(),
            language_library_search_path: self.language_library_search_path.clone(),
            nodes_to_check: self.nodes_to_check.clone(),
            nodes_to_query: self
                .nodes_to_check
                .iter()
                .map(|s| ::tree_sitter::Query::new(language, &s).unwrap())
                .collect(),
            language,
        };

        Ok(language_sitter)
    }
}

impl LanguageSitterParsers for LanguageSitters {
    fn is_initialised(&self, language: &str) -> bool {
        self.language_parsers_uninitialised.contains_key(language)
    }

    fn initialise(&mut self, language: &str) -> Result<()> {
        let language_sitter = self.language_parsers_initialised.get(language);
        if language_sitter.is_none() {
            let uninitialised = self.language_parsers_uninitialised.get(language);
            if uninitialised.is_none() {
                return Result::Err(anyhow::anyhow!(
                    "Unable to find tree sitter library for language {}",
                    language
                ));
            }
            let language_sitter_initialised = uninitialised.unwrap().initialise()?;
            self.language_parsers_initialised
                .insert(language.to_string(), language_sitter_initialised);
        }
        Ok(())
    }

    fn parse_str<'a>(&self, language: &'a str, s: &'a str) -> Result<Vec<LanguageSitterResult>> {
        let Some(language_sitter) = self.language_parsers_initialised.get(language) else {
            return anyhow::Result::Err(anyhow::anyhow!(
                "Language \"{}\" not initialised",
                language
            ));
        };
        language_sitter.parse_str(s)
    }
}

impl LanguageSitterParser for LanguageSitterInitialised {
    fn parse_str<'a>(&self, s: &'a str) -> Result<Vec<LanguageSitterResult>> {
        let mut parser = Parser::new();
        parser.set_language(self.language)?;

        let Some(tree) = parser.parse(s, None) else {
            return anyhow::Result::Err(anyhow::anyhow!(
                "Error parsing. \"{}\" tree sitter did not return a tree",
                self.language_name
            ));
        };

        let root_node = tree.root_node();
        let mut sbytes = s.as_bytes();
        let mut result = Vec::<LanguageSitterResult>::new();

        let mut query_cursor = ::tree_sitter::QueryCursor::new();

        for query in &self.nodes_to_query {
            query_cursor
                .captures(query, root_node, sbytes)
                .for_each(|c| {
                    println!("Capture test: {:?}", c);
                    c.0.captures.into_iter().for_each(|cap| {
                        sbytes.text(cap.node).for_each(|deep| {
                            result.push(LanguageSitterResult {
                                text: std::str::from_utf8(deep).unwrap().to_string(),
                                start_pos: cap.node.start_byte(),
                                end_pos: cap.node.end_byte(),
                            });
                            // comments.push(std::str::from_utf8(deep).unwrap().to_string());
                        })
                    });
                });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_rust_parsing() {
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
        let language_sitter = LanguageSitterUninitialised::new(
            "rust",
            &LanguageSitterConfigNode {
                language_library_name: "libtree-sitter-rust".to_string(),
                file_extensions: ["rs".to_string(), "ltlsp".to_string()].to_vec(),
                expressions: [
                    "(line_comment) @line".to_string(),
                    "(block_comment) @block".to_string(),
                ]
                .to_vec(),
            },
        )
        .unwrap()
        .initialise()
        .unwrap();
        let comments = language_sitter.parse_str(rust).unwrap();

        for i in comments {
            info!("Comment: {}", i.text);
            // info!("{:?}", i);
        }
    }
    #[test]
    fn test_parser() {
        let _ = crate::test_utils::setup_tracing();
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Error loading Rust grammar");
        let source_code = "fn test() {}";
        let tree = parser.parse(source_code, None).unwrap();
        let root_node = tree.root_node();

        assert_eq!(root_node.kind(), "source_file");
        assert_eq!(root_node.start_position().column, 0);
        assert_eq!(root_node.end_position().column, 12);
    }

    #[test]
    fn test_rust_identify_comments() {
        let _ = crate::test_utils::setup_tracing();
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_rust::language())
            .expect("Error loading Rust grammar");
        let source_code = r###"""
        /// This is a comment..
        fn test() {}
        """###;
        let tree = parser.parse(source_code, None).unwrap();
        let root_node = tree.root_node();

        println!("{:?}", root_node);
        println!("{:?}", root_node.to_sexp());
        let mut cursor = root_node.walk();
        for i in root_node.children(&mut cursor) {
            println!("{:?}", i);
        }
        assert_eq!(root_node.kind(), "source_file");
        assert_eq!(root_node.start_position().column, 0);
        assert_eq!(root_node.end_position().column, 10);
    }
    #[test]
    fn test_markdown_identify_text() {
        let _ = crate::test_utils::setup_tracing();
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_md::language())
            .expect("Error loading Rust grammar");
        let source_code = r###"""# Heading
        Some text with a [https://github.com/](Github)
        """###;
        let tree = parser.parse(source_code, None).unwrap();
        let root_node = tree.root_node();

        println!("{:?}", root_node);
        println!("{:?}", root_node.to_sexp());
        let mut cursor = root_node.walk();
        for i in root_node.children(&mut cursor) {
            println!("{:?}", i);
        }
        assert_eq!(root_node.kind(), "document");
        assert_eq!(root_node.start_position().column, 0);
        assert_eq!(root_node.end_position().column, 10);
    }
}
