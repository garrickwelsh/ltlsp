use anyhow::Result;
use std::path::PathBuf;

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

pub(crate) fn prioritise_config_dirs() -> Result<Vec<PathBuf>> {
    let mut retval = Vec::<PathBuf>::new();

    // Get config directory
    let _strategy = app_strategy::choose_app_strategy(etcetera::AppStrategyArgs {
        top_level_domain: "develop".to_string(),
        author: "Garrick".to_string(),
        app_name: "ltlsp".to_string(),
    })?;
    retval.push(_strategy.config_dir());

    // Get cargo directory
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_dir = PathBuf::from(dir);
        let path = manifest_dir;
        // Once we release should it below in another dir?
        // let path = manifest_dir.parent().unwrap();
        retval.push(path.to_path_buf());
    }

    // If install via cargo it will go another directory, so we'd want to get the parent.
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_dir = PathBuf::from(dir);
        let path = manifest_dir.parent().unwrap();
        retval.push(path.to_path_buf());
    }

    if let Ok(dir) = std::env::current_exe() {
        let path = std::fs::canonicalize(dir)?;
        let path = path.parent().unwrap().to_path_buf();
        retval.push(path);
    }

    Ok(retval)
}

pub(crate) fn prioritise_runtime_grammar_dirs() -> Result<Vec<PathBuf>> {
    let result = prioritise_config_dirs()?
        .into_iter()
        .map(|pb: PathBuf| pb.join("runtime").join("ltlsp_grammars"))
        .collect();
    Ok(result)
}

pub(crate) fn get_tree_sitter_config() -> Result<LanguageSitterConfig> {
    let config_dirs = prioritise_config_dirs()?;
    let mut config_file_location = Option::None;
    for i in config_dirs {
        let config_file_exists = i.join("runtime").join("languages.toml");
        if config_file_exists.exists() {
            config_file_location = Some(config_file_exists);
        }
    }

    if let Some(config_file_location) = config_file_location {
        let config_data = std::fs::read_to_string(config_file_location).unwrap();
        toml::from_str(&config_data).map_err(anyhow::Error::msg)
    } else {
        anyhow::bail!("Unable to find languages.toml config file.")
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
