/// Knowledge is power
fn main() -> anyhow::Result<()> {
    cli_log::init_cli_log!();
    bacon::run()?;
    cli_log::info!("bye");
    Ok(())
}
