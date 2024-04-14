use serde::Deserialize;
use tree_sitter::{Language, Parser};

#[derive(Deserialize, Debug)]
struct TreeSitterLanguage {
    language: String,
    source: String,

    nodes: Vec<String>,
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
