mod configuration;

use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use tree_sitter::Node;
use tree_sitter::TextProvider;
use tree_sitter::Tree;
use tree_sitter::{Language, Parser};

#[derive(Deserialize, Debug)]
struct TreeSitterLanguage {
    language: String,
    source: String,

    nodes: Vec<String>,
}

#[cfg(unix)]
const DYLIB_EXTENSION: &str = "so";

#[cfg(windows)]
const DYLIB_EXTENSION: &str = "dll";

#[cfg(target_arch = "wasm32")]
const DYLIB_EXTENSION: &str = "wasm";

pub(crate) fn get_language(name: &str) -> Result<Language> {
    use std::path::PathBuf;

    use libloading::{Library, Symbol};
    let mut rel_library_path = PathBuf::new().join("grammars").join(name);
    rel_library_path.set_extension(DYLIB_EXTENSION);
    let library_path = PathBuf::from("/home/gaz/devel/ltlsp").join(rel_library_path);

    let library = unsafe { Library::new(&library_path) }
        .with_context(|| format!("Error opening dynamic library {:?}", library_path))?;
    let language_fn_name = format!("tree_sitter_{}", name.replace('-', "_"));
    let language = unsafe {
        let language_fn: Symbol<unsafe extern "C" fn() -> Language> = library
            .get(language_fn_name.as_bytes())
            .with_context(|| format!("Failed to load symbol {}", language_fn_name))?;
        language_fn()
    };
    std::mem::forget(library);
    Ok(language)
}

pub(crate) fn parse_rust(file_contents: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_rust::language())
        .expect("Error loading language");
    let tree = parser.parse(file_contents, None).unwrap();
    tree
}

pub(crate) fn get_comments(tree: &Tree, file_contents: &str) -> Vec<String> {
    let mut comments = Vec::<String>::new();
    let root_node = tree.root_node();
    let file_bytes = file_contents.as_bytes();
    get_comments_from_node(&mut comments, root_node, file_bytes);
    comments
}

fn get_comments_from_node(comments: &mut Vec<String>, node: Node, file_bytes: &[u8]) {
    let mut query_cursor = tree_sitter::QueryCursor::new();
    let query =
        tree_sitter::Query::new(tree_sitter_rust::language(), "(line_comment) @line").unwrap();
    let mut bts = file_bytes;
    query_cursor
        .captures(&query, node, file_bytes)
        // query.capture_names().into_iter().for_each(|c| {
        .for_each(|c| {
            println!("Capture test: {:?}", c);
            c.0.captures.into_iter().for_each(|cap| {
                bts.text(cap.node)
                    .for_each(|deep| comments.push(std::str::from_utf8(deep).unwrap().to_string()))
            });
        });
}

pub(crate) trait LanguageSitterParser {
    fn parse_str<'a>(&self, s: &'a str) -> Vec<LanguageSitterResult>;
}

#[derive(Debug)]
pub(crate) struct LanguageSitterResult {}

#[derive(Debug)]
pub(crate) struct LanguageSitterImpl {
    language: Language,
}

impl LanguageSitterParser for LanguageSitterImpl {
    fn parse_str<'a>(&self, s: &'a str) -> Vec<LanguageSitterResult> {
        let mut parser = Parser::new();
        parser.set_language(self.language).unwrap();
        let tree = parser.parse(s, None).unwrap();
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_rust_parsing() {
        let rust = r###"
 // This is a comment, and is ignored by the compiler.
// You can test this code by clicking the "Run" button over there ->
// or if you prefer to use your keyboard, you can use the "Ctrl + Enter"
// shortcut.

// This code is editable, feel free to hack it!
// You can always return to the original code by clicking the "Reset" button ->

//! This is the main function.
fn main() {
    // Statements here are executed when the compiled binary is called.

    // Print text to the console.
    println!("Hello World!");
}
"###;
        let tree = parse_rust(rust);
        let root_node = tree.root_node();

        assert_eq!(root_node.kind(), "source_file");
        println!("{:?}", root_node.to_sexp());
        let comments = get_comments(&tree, rust);
        println!("Comments vec - {:?}", comments);
        for i in comments {
            println!("{:?}", i);
        }
    }

    #[test]
    fn test_toml_configuration() {
        let config: TreeSitterLanguage = toml::from_str(
            r#"
   language = 'markdown'
   source = 'some source'

   nodes = ["node1", "node2"]
   
"#,
        )
        .unwrap();
        assert_eq!(config.language, "markdown");
    }

    #[test]
    fn test_parser() {
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
