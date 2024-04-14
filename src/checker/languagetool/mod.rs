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

pub(crate) struct LanguageToolRunner {
    server: String,
    port: u32,
    initialisation: LanguageToolInitialisation,
}

impl LanguageToolRunner {
    /// Startup language tool if it's not already running.
    pub(crate) async fn initialise_language_tool(server: &str, port: u32) -> LanguageToolRunner {
        if check_if_languagetool_up().await {
            info!("languagetool already running :)");
            return LanguageToolRunner {
                server: server.to_owned(),
                port,
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
                return LanguageToolRunner {
                    server: server.to_owned(),
                    port,
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

impl Drop for LanguageToolRunner {
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

    #[tokio::test]
    async fn start_language_tool() -> Result<(), Box<dyn std::error::Error>> {
        let subscriber = tracing_subscriber::fmt()
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .with_test_writer()
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
        let _ = LanguageToolRunner::initialise_language_tool("test", 0).await;
        Ok(())
    }
    #[ignore]
    #[tokio::test]
    async fn query_language_tool() -> Result<(), Box<dyn std::error::Error>> {
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
        Ok(())
    }

    #[tokio::test]
    async fn query_language_tool_with_serde() -> Result<(), Box<dyn std::error::Error>> {
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
