use std::error::Error;

use tracing::info;

mod checker;
mod lsp_server;
mod tree_sitter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    tracing_subscriber::fmt::init();
    info!("Hello, world!");
    Ok(())
}
