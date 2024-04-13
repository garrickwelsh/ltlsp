use std::{error::Error, fs::OpenOptions};

use tracing::info;

mod checker;
mod lsp_server;
mod tree_sitter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    let log_file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .open("/home/gaz/test.log")?;
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_writer(log_file)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    info!("Hello, world!");
    Ok(())
}
