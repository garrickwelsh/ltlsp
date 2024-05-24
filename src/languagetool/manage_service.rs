use std::{collections::HashMap, io::ErrorKind};

use tokio::process::{Child, Command};
use tracing::{error, info};

use super::{LanguageToolRequest, LanguageToolRequestBuilder};

#[derive(Copy, Clone)]
pub(crate) enum _ContainerType {
    _PodMan,
    _Docker,
    _Incus,
}

pub(crate) enum LanguageToolInitialisation {
    AlreadyRunning,
    LocalExecutable(Child),
    Container(_ContainerType, Child),
}

pub(crate) struct LanguageToolRunnerRemote {
    pub(crate) server: String,
    pub(crate) port: u16,
    pub(crate) language: String,
}

pub(crate) struct LanguageToolRunnerLocal {
    pub(crate) port: u16,
    pub(crate) language: String,
    pub(crate) initialisation: LanguageToolInitialisation,
}

pub(crate) trait LanguageToolRunner {
    fn server(&self) -> &str;
    fn port(&self) -> u16;
    fn language(&self) -> &str;
    fn new_request(&self) -> impl LanguageToolRequestBuilder;
}

impl LanguageToolRunner for LanguageToolRunnerRemote {
    fn server(&self) -> &str {
        &self.server
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn language(&self) -> &str {
        &self.language
    }

    fn new_request(&self) -> impl LanguageToolRequestBuilder {
        LanguageToolRequest::new(&self.server, self.port, &self.language)
    }
}

impl LanguageToolRunner for LanguageToolRunnerLocal {
    fn server(&self) -> &str {
        "localhost"
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn language(&self) -> &str {
        &self.language
    }

    fn new_request(&self) -> impl LanguageToolRequestBuilder {
        LanguageToolRequest::new(self.server(), self.port(), self.language())
    }
}

/// Make a request to language tool. If a good response is not received send an error.
async fn check_if_languagetool_up<'a>(server: &'a str, port: u16, language: &'a str) -> bool {
    let client = reqwest::Client::new();
    let mut form = HashMap::new();
    form.insert("language", language);
    form.insert(
        "text",
        "This is some somple test text. I'm hoping that language till tool will understand it.",
    );

    let mut url = reqwest::Url::parse("http://server:80/v2/check").unwrap();
    url.set_host(Some(server)).unwrap();
    url.set_port(Some(port)).unwrap();
    let res = client.post(url).form(&form).send().await;
    res.is_ok()
}

impl LanguageToolRunnerRemote {
    /// Startup language tool if it's not already running.
    pub(crate) async fn initialise_language_tool(
        server: &str,
        port: u16,
        language: &str,
    ) -> impl LanguageToolRunner {
        if check_if_languagetool_up(server, port, language).await {
            info!("languagetool already running :)");
            return LanguageToolRunnerRemote {
                server: server.to_string(),
                port,
                language: language.to_string(),
            };
        }
        todo!();
    }
}

impl<'a> LanguageToolRunnerLocal {
    /// Startup language tool if it's not already running.
    pub(crate) async fn initialise_language_tool_local(
        port: u16,
        language: &str,
    ) -> LanguageToolRunnerLocal {
        if check_if_languagetool_up("localhost", port, language).await {
            info!("languagetool already running :)");
            return LanguageToolRunnerLocal {
                port,
                language: language.to_string(),
                initialisation: LanguageToolInitialisation::AlreadyRunning,
            };
        }
        match Command::new("languagetool")
            .args(&["--http", "--port", &format!("{}", port)])
            .kill_on_drop(true)
            .spawn()
        {
            Ok(child) => {
                info!("languagetool Was spawned :)");
                return LanguageToolRunnerLocal {
                    port,
                    language: language.to_string(),
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
        let container_types = [
            _ContainerType::_PodMan,
            _ContainerType::_Docker,
            _ContainerType::_Incus,
        ];
        for container_type in container_types {
            let cmd = match container_type {
                _ContainerType::_PodMan => "podman",
                _ContainerType::_Docker => "docker",
                _ContainerType::_Incus => "incus",
            };
            match Command::new(cmd)
                .args(&[
                    "run",
                    "--name=lt",
                    "ghcr.io/garrickwelsh/languagetool:latest",
                ])
                .kill_on_drop(false)
                .spawn()
            {
                Ok(child) => {
                    info!("languagetool Was spawned : via {})", cmd);
                    return LanguageToolRunnerLocal {
                        port,
                        language: language.to_string(),
                        initialisation: LanguageToolInitialisation::Container(
                            container_type,
                            child,
                        ),
                    };
                }
                Err(e) => {
                    if e.kind() == ErrorKind::NotFound {
                        info!(
                            "`{}` was not found in path! Attempting other containers!",
                            cmd
                        )
                    } else {
                        info!("Some strange error occurred attempting to start container!");
                    }
                }
            }
        }
        panic!("No mechanism to start language tool was found");
    }

    pub(crate) async fn initialise_language_tool(
        port: u16,
        language: &str,
    ) -> impl LanguageToolRunner {
        LanguageToolRunnerLocal::initialise_language_tool_local(port, language).await
    }
}

impl Drop for LanguageToolRunnerLocal {
    fn drop(&mut self) {
        match self.initialisation {
            LanguageToolInitialisation::LocalExecutable(_) => {
                info!("Languagetool should be killed on drop")
            }
            LanguageToolInitialisation::Container(container_type, _) => {
                let cmd = match container_type {
                    _ContainerType::_PodMan => "podman",
                    _ContainerType::_Docker => "docker",
                    _ContainerType::_Incus => "incus",
                };
                // Command isn't appropriate to stop podman
                match Command::new(cmd).args(&["rm", "-f", "lt"]).spawn() {
                    Ok(mut child) => {
                        child.wait();
                        info!("languagetool shutdown attempt : via {})", cmd);
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::NotFound {
                            info!(
                                "`{}` was not found in path! Attempting other containers!",
                                cmd
                            )
                        } else {
                            info!("Some strange error occurred attempting to start container!");
                        }
                    }
                }
                info!("Stop {}", cmd);
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

    use crate::{languagetool::manage_service::LanguageToolRunnerLocal, test_utils::setup_tracing};

    #[tokio::test]
    async fn start_language_tool() -> Result<(), Box<dyn std::error::Error>> {
        setup_tracing()?;
        let _ = LanguageToolRunnerLocal::initialise_language_tool(8081, "en-AU").await;
        Ok(())
    }

    #[tokio::test]
    async fn query_language_tool() -> Result<(), Box<dyn std::error::Error>> {
        setup_tracing()?;
        let lt = LanguageToolRunnerLocal::initialise_language_tool(8081, "en-AU").await;
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
}
