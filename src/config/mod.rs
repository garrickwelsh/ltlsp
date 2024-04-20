use etcetera::{app_strategy, AppStrategy};
use serde::Deserialize;

// TODO: Build configurations to extract specific node types from treesitter for
// consumption by language tool.
// More advanced features will be to remove excess comment tags.

#[derive(Debug, Deserialize)]
pub(crate) struct LanguageSitterConfig<'a> {
    _language: &'a str,
    _library_location: &'a str,

    _nodes: Vec<LanguageSitterConfigNode<'a>>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LanguageSitterConfigNode<'a> {
    _name: &'a str,
    _expresson: &'a str,
}

pub(crate) fn get_config<'a>(_language: &str) -> LanguageSitterConfig<'a> {
    let _strategy = app_strategy::choose_app_strategy(etcetera::AppStrategyArgs {
        top_level_domain: "develop".to_string(),
        author: "Garrick".to_string(),
        app_name: "ltlsp".to_string(),
    })
    .unwrap();
    let _config_dir = _strategy.config_dir();
    LanguageSitterConfig {
        _language: "rust",
        _library_location: "TODO",
        _nodes: Vec::<LanguageSitterConfigNode>::new(),
    }
}

#[cfg(test)]
mod test {
    use super::LanguageSitterConfig;

    #[test]
    fn test_toml_configuration() {
        let config: LanguageSitterConfig = toml::from_str(
            r#"
   language = 'markdown'
   source = 'some source'

   nodes = ["node1", "node2"]
   
"#,
        )
        .unwrap();
        assert_eq!(config.language, "markdown");
    }
}
