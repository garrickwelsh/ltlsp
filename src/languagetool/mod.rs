use std::{collections::HashMap, future::Future, io::ErrorKind};

use reqwest;
use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};
use tracing::{error, info};

/// Make a request to language tool. If a good response is not received send an error.
async fn check_if_languagetool_up() -> bool {
    let client = reqwest::Client::new();
    let mut form = HashMap::new();
    form.insert("language", "en-AU");
    form.insert(
        "text",
        "This is some somple test text. I'm hoping that language till tool will understand it.",
    );

    let res = client
        .post("http://localhost:8081/v2/check")
        .form(&form)
        .send()
        .await;
    res.is_ok()
}

pub(crate) enum ContainerType {
    PodMan,
    Docker,
    Lxc,
}

pub(crate) enum LanguageToolInitialisation {
    AlreadyRunning,
    LocalExecutable(Child),
    Container(ContainerType),
}

pub(crate) struct LanguageToolRunnerImpl<'a> {
    server: &'a str,
    port: u32,
    language: &'a str,
    initialisation: LanguageToolInitialisation,
}

pub(crate) struct LanguageToolRequestImpl<'a> {
    server: &'a str,
    port: u32,
    language: &'a str,
    document_data: LanguageToolDocumentData<'a>,
}

pub(crate) trait LanguageToolRequest<'a> {
    fn add_text(&mut self, text: &'a str);
    fn add_markup(&mut self, markup: &'a str);
    async fn execute_request(&self) -> Result<String, reqwest::Error>;
}
pub(crate) trait LanguageTool<'a> {
    fn new_request(&self) -> impl LanguageToolRequest<'a>;
}

impl<'a> LanguageToolRequest<'a> for LanguageToolRequestImpl<'a> {
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

impl<'a> LanguageTool<'a> for LanguageToolRunnerImpl<'a> {
    fn new_request(&self) -> impl LanguageToolRequest<'a> {
        LanguageToolRequestImpl::new(self.server, self.port, self.language)
    }
}

impl<'a> LanguageToolRequestImpl<'a> {
    fn new(server: &'a str, port: u32, language: &'a str) -> LanguageToolRequestImpl<'a> {
        LanguageToolRequestImpl {
            server,
            port,
            language,
            document_data: LanguageToolDocumentData {
                annotation: Vec::<LanguageToolText>::new(),
            },
        }
    }
}

impl<'a> LanguageToolRunnerImpl<'a> {
    /// Startup language tool if it's not already running.
    pub(crate) async fn initialise_language_tool(
        server: &'a str,
        port: u32,
        language: &'a str,
    ) -> impl LanguageTool<'a> {
        if check_if_languagetool_up().await {
            info!("languagetool already running :)");
            return LanguageToolRunnerImpl {
                server,
                port,
                language,
                initialisation: LanguageToolInitialisation::AlreadyRunning,
            };
        }

        match Command::new("languagetool")
            .args(&["--http"])
            .kill_on_drop(true)
            .spawn()
        {
            Ok(child) => {
                info!("languagetool Was spawned :)");
                return LanguageToolRunnerImpl {
                    server,
                    port,
                    language: "en-AU",
                    initialisation: LanguageToolInitialisation::LocalExecutable(child),
                };
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    info!("`languagetool` was not found in path! Attempting to start container!")
                } else {
                    info!("Some strange error occurred :( Attempt to start container!");
                }
            }
        }
        error!("language tool container start is not yet supported.");
        todo!("Not yet implemented.");
    }
}

impl<'a> Drop for LanguageToolRunnerImpl<'a> {
    fn drop(&mut self) {
        match self.initialisation {
            LanguageToolInitialisation::LocalExecutable(_) => {
                info!("Languagetool should be killed on drop")
            }
            LanguageToolInitialisation::Container(ContainerType::PodMan) => {
                todo!("Stop podman");
            }
            LanguageToolInitialisation::Container(ContainerType::Docker) => {
                todo!("Stop docker");
            }
            LanguageToolInitialisation::Container(ContainerType::Lxc) => {
                todo!("Stop Lxc");
            }
            LanguageToolInitialisation::AlreadyRunning => {
                info!("Languagetool was already running no need to shut it down");
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;
    use tokio;

    fn setup_tracing() -> Result<(), Box<dyn std::error::Error>> {
        let subscriber = tracing_subscriber::fmt()
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .with_test_writer()
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
        Ok(())
    }

    #[tokio::test]
    async fn start_language_tool() -> Result<(), Box<dyn std::error::Error>> {
        setup_tracing()?;
        let _ = LanguageToolRunnerImpl::initialise_language_tool("test", 0, "en-AU").await;
        Ok(())
    }

    #[tokio::test]
    async fn query_language_tool() -> Result<(), Box<dyn std::error::Error>> {
        setup_tracing()?;
        let lt = LanguageToolRunnerImpl::initialise_language_tool("", 0, "en-AU").await;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        let client = reqwest::Client::new();

        let mut form = HashMap::new();
        form.insert("language", "en-AU");
        form.insert(
            "text",
            "This is some somple test text. I'm hoping that language till tool will understand it.",
        );

        let res = client
            .post("http://localhost:8081/v2/check")
            .form(&form)
            .send()
            .await?;
        println!("{:?}", res);
        println!("{:?}", res.text().await?);
        drop(lt);
        Ok(())
    }

    #[tokio::test]
    async fn query_language_tool_with_serde() -> Result<(), Box<dyn std::error::Error>> {
        setup_tracing()?;
        let lt = LanguageToolRunnerImpl::initialise_language_tool("", 0, "en-AU").await;
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
        setup_tracing()?;
        let lt = LanguageToolRunnerImpl::initialise_language_tool("localhost", 8081, "en-AU").await;
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

// #[derive(Serialize, Debug)]
// pub(crate) struct LanguageToolRequest<'a> {
//     language: &'a str,
//     data: LanguageToolDocumentData<'a>,
// }

// impl LanguageToolRequest<'_> {
//     fn new<'a>() -> LanguageToolRequest<'a> {
//         LanguageToolRequest {
//             language: "en-AU",
//             data: LanguageToolDocumentData {
//                 annotation: Vec::<LanguageToolText>::new(),
//             },
//         }
//     }
// }
