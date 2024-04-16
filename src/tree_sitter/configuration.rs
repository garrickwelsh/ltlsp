use serde::Deserialize;

// TODO: Build configurations to extract specific node types from treesitter for
// consumption by language tool.
// More advanced features will be to remove excess comment tags.

#[derive(Debug, Deserialize)]
pub(crate) struct LanguageSitterConfiguration<'a> {
    language: &'a str,
    library_location: &'a str,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LanguageSitterConfigurationNode<'a> {
    name: &'a str,
    expresson: &'a str,
}

pub(crate) fn get_configuration<'a>(_language: &str) -> LanguageSitterConfiguration<'a> {
    LanguageSitterConfiguration {
        language: "rust",
        library_location: "TODO",
    }
}
