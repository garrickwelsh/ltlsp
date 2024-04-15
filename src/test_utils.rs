pub(crate) fn setup_tracing() -> Result<(), Box<dyn std::error::Error>> {
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
