use etcetera::{app_strategy, AppStrategy};
use serde::Deserialize;
use tracing::info;

// TODO: Build configurations to extract specific node types from treesitter for
// consumption by language tool.
// More advanced features will be to remove excess comment tags.

#[derive(Debug, Deserialize)]
pub(crate) struct LanguageSitterConfig {
    _language: String,
    _library_location: String,

    _nodes: Vec<LanguageSitterConfigNode>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LanguageSitterConfigNode {
    _name: String,
    _expresson: String,
}

pub(crate) fn get_config(_language: &str) -> LanguageSitterConfig {
    let _strategy = app_strategy::choose_app_strategy(etcetera::AppStrategyArgs {
        top_level_domain: "develop".to_string(),
        author: "Garrick".to_string(),
        app_name: "ltlsp".to_string(),
    })
    .unwrap();
    let _config_dir = _strategy.config_dir();
    info!("config dir is: {}", _config_dir.display());
    LanguageSitterConfig {
        _language: "rust".to_string(),
        _library_location: "TODO".to_string(),
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
        assert_eq!(config._language, "markdown");
    }
}
