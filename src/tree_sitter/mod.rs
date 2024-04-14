use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
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

pub fn get_language(name: &str) -> Result<Language> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
