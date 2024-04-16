use tracing::error;

pub(crate) fn setup_tracing() -> Result<(), Box<dyn std::error::Error>> {
    let result = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_test_writer()
        .try_init();
    if result.is_err() {
        error!("Tracing likely already initialised, ignoring initialisation attempt.");
    }
    Ok(())
}
