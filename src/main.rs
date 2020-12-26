mod app;
mod args;
mod cli;
mod command_output;
mod command_result;
mod default_package_config;
mod default_prefs;
mod drawing;
mod executor;
mod failure;
mod job;
mod line;
mod line_analysis;
mod line_type;
mod mission;
mod package_config;
mod prefs;
mod report;
mod scroll;
mod settings;
mod state;
mod stats;
mod tty;
mod wrap;

pub use {
    app::*,
    args::*,
    cli::*,
    command_output::*,
    command_result::*,
    default_package_config::*,
    default_prefs::*,
    drawing::*,
    executor::*,
    failure::*,
    job::*,
    line::*,
    line_analysis::*,
    line_type::*,
    mission::*,
    package_config::*,
    prefs::*,
    report::*,
    scroll::*,
    settings::*,
    state::*,
    stats::*,
    tty::*,
    wrap::*,
};

#[macro_use]
extern crate log;

use {
    anyhow::*,
    log::LevelFilter,
    simplelog,
    std::{env, fs::File, str::FromStr},
};

/// configure the application log according to env variable.
///
/// There's no log unless the BACON_LOG environment variable is set to
///  a valid log level (trace, debug, info, warn, error, off)
/// Example:
///      BACON_LOG=info broot
/// As bacon is a terminal application, we only log to a file (bacon.log)
fn configure_log() {
    let level = env::var("BACON_LOG").unwrap_or_else(|_| "off".to_string());
    if level == "off" {
        return;
    }
    if let Ok(level) = LevelFilter::from_str(&level) {
        simplelog::WriteLogger::init(
            level,
            simplelog::Config::default(),
            File::create("bacon.log").expect("Log file can't be created"),
        )
        .expect("log initialization failed");
        info!(
            "Starting bacon v{} with log level {}",
            env!("CARGO_PKG_VERSION"),
            level
        );
    }
}

/// Knowledge is power
fn main() -> Result<()> {
    configure_log();
    cli::run()?;
    info!("bye");
    Ok(())
}
