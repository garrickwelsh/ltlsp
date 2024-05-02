pub(crate) mod manage_service;

use std::collections::HashMap;

use reqwest;
use serde::Serialize;
use tracing::info;

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
    async fn execute_request(&self) -> Result<String, reqwest::Error>;
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

    async fn execute_request(&self) -> Result<String, reqwest::Error> {
        let request_data = serde_json::to_string_pretty(&self.document_data).unwrap();
        let mut map = HashMap::<&str, &str>::new();
        map.insert("language", self.language);
        map.insert("data", &request_data);
        let url = format!("http://{}:{}/v2/check", self.server, self.port);
        let client = reqwest::Client::new();
        let res = client.post(url).form(&map).send().await?;
        let result = res.text().await?;
        Ok(result)
    }
}

pub(crate) struct LanguageTool<'a> {
    runner: LanguageToolRunnerRemote<'a>,
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
        info!(result);
        drop(request);
        drop(lt);
        Ok(())
    }
}
