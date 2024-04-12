use tracing::info;

mod checker;
mod tree_sitter;

fn main() {
    tracing_subscriber::fmt::init();
    info!("Hello, world!");
}
