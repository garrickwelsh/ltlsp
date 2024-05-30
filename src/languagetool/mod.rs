pub(crate) mod manage_service;

use std::collections::HashMap;

use reqwest;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::tree_sitter::LanguageSitterResult;

use self::manage_service::LanguageToolRunnerRemote;

pub(crate) struct LanguageToolRequest<'a> {
    server: &'a str,
    port: u16,
    language: &'a str,
    document_data: LanguageToolDocumentData<'a>,
}

pub(crate) trait LanguageToolRequestBuilder<'a> {
    fn add_text(&mut self, text: &'a str);
    fn add_markup(&mut self, markup: &'a str);
    async fn execute_request(&self) -> Result<LanguageToolResult, reqwest::Error>;
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageToolResult {
    pub(crate) software: LanguageToolResultSoftware,
    pub(crate) language: LanguageToolResultLanguage,
    pub(crate) matches: Vec<LanguageToolResultMatch>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageToolResultSoftware {
    name: String,
    version: String,
    build_date: String,
    api_version: i32,
    status: String,
    premium: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageToolResultLanguage {
    name: String,
    code: String,
    detected_language: LanguageToolResultLanguageDetected,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageToolResultLanguageDetected {
    name: String,
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageToolResultMatch {
    pub(crate) message: String,
    pub(crate) short_message: String,
    pub(crate) offset: i32,
    pub(crate) length: i32,
    pub(crate) sentence: String,
    pub(crate) replacements: Vec<LanguageToolResultListItem>,
    pub(crate) context: LanguageToolResultMatchContext,
    pub(crate) rule: LanguageToolResultMatchRule,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageToolResultListItem {
    pub(crate) value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageToolResultMatchContext {
    text: String,
    offset: i32,
    length: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LanguageToolResultMatchRule {
    pub(crate) id: String,
    pub(crate) sub_id: Option<String>,
    pub(crate) description: String,
    pub(crate) urls: Option<Vec<LanguageToolResultListItem>>,
    pub(crate) issue_type: String,
    pub(crate) category: LanguageToolResultMatchRuleCategory,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LanguageToolResultMatchRuleCategory {
    id: String,
    name: String,
}

#[derive(Serialize, Debug)]
pub(crate) struct LanguageToolDocumentData<'a> {
    annotation: Vec<LanguageToolText<'a>>,
}

#[derive(Serialize, Debug)]
pub(crate) enum LanguageToolText<'a> {
    #[serde(rename = "text")]
    Text(&'a str),
    #[serde(rename = "markup")]
    Markup(&'a str),
}

impl<'a> LanguageToolRequestBuilder<'a> for LanguageToolRequest<'a> {
    fn add_text(&mut self, text: &'a str) {
        self.document_data
            .annotation
            .push(LanguageToolText::Text(text));
    }

    fn add_markup(&mut self, markup: &'a str) {
        self.document_data
            .annotation
            .push(LanguageToolText::Markup(markup));
    }

    async fn execute_request(&self) -> Result<LanguageToolResult, reqwest::Error> {
        let request_data = serde_json::to_string_pretty(&self.document_data).unwrap();
        let mut map = HashMap::<&str, &str>::new();
        map.insert("language", self.language);
        map.insert("data", &request_data);
        let url = format!("http://{}:{}/v2/check", self.server, self.port);
        let client = reqwest::Client::new();
        let res = client.post(url).form(&map).send().await?;
        let result = res.json::<LanguageToolResult>().await?;
        debug!("{:?}", result);
        Ok(result)
    }
}

pub(crate) struct LanguageTool {
    runner: LanguageToolRunnerRemote,
}

impl<'a> LanguageToolRequest<'a> {
    fn new(server: &'a str, port: u16, language: &'a str) -> LanguageToolRequest<'a> {
        LanguageToolRequest {
            server,
            port,
            language,
            document_data: LanguageToolDocumentData {
                annotation: Vec::<LanguageToolText>::new(),
            },
        }
    }
}

impl<'a> From<LanguageSitterResult> for LanguageToolDocumentData<'a> {
    fn from(value: LanguageSitterResult) -> Self {
        todo!();
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use crate::config::LanguageSitterConfigNode;
    use crate::tree_sitter::{LanguageSitterParser, LanguageSitterUninitialised};

    use super::super::test_utils::setup_tracing;
    use super::LanguageToolRequestBuilder;
    use super::*;
    use tests::manage_service::LanguageToolRunnerLocal;
    use tokio;

    #[tokio::test]
    async fn query_language_tool_with_serde() -> Result<(), Box<dyn std::error::Error>> {
        setup_tracing()?;
        let lt = LanguageToolRunnerLocal::initialise_language_tool(8081, "en-AU").await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let client = reqwest::Client::new();

        let mut request = HashMap::new();
        request.insert("language", "en-AU");
        println!("request created");

        let mut data = LanguageToolDocumentData {
            annotation: Vec::<LanguageToolText>::new(),
        };
        data.annotation.push(LanguageToolText::Markup("<h1>"));
        data.annotation.push(LanguageToolText::Text(
            "Here is som text that I'd like spell checked.",
        ));
        data.annotation.push(LanguageToolText::Text(
            "Is this something you're able to help me with?",
        ));
        data.annotation.push(LanguageToolText::Markup("</h1>"));

        let request_data = serde_json::to_string_pretty(&data).unwrap();
        request.insert("data", &request_data);

        println!("{}", &request_data);

        println!("request populated with data");
        let res = client
            .post("http://localhost:8081/v2/check")
            .form(&request)
            .send()
            .await;
        println!("{:?}", res);
        let res = res?;
        println!("{:?}", res);
        println!("{:?}", res.text().await?);
        drop(lt);
        Ok(())
    }

    #[tokio::test]
    async fn query_language_tool_with_wrapping_implementation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::languagetool::manage_service::LanguageToolRunner;
        setup_tracing()?;
        let lt = LanguageToolRunnerLocal::initialise_language_tool(8081, "en-AU").await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let mut request = lt.new_request();
        request.add_markup("<h1>");
        request.add_text("Here is som text that I'd like spell checked.");
        request.add_text("Is this something you're able to help me with?");
        request.add_markup("</h1>");
        let result = request.execute_request().await?;
        info!("{:?}", result);
        drop(request);
        drop(lt);
        Ok(())
    }

    #[tokio::test]
    async fn query_language_tool_with_data_from_treesitter(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::languagetool::manage_service::LanguageToolRunner;
        setup_tracing()?;
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
                language_fn_name: "tree_sitter_rust".to_string(),
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
        let lt_comments = comments
            .iter()
            .map(|c| c.text.clone())
            .collect::<Vec<String>>();
        let lt = LanguageToolRunnerLocal::initialise_language_tool(8081, "en-AU").await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let mut request = lt.new_request();
        for c in &lt_comments {
            request.add_text(c);
        }
        let result = request.execute_request().await?;
        info!("{:?}", result);
        drop(comments);
        drop(request);
        drop(lt);
        Ok(())
    }
}
