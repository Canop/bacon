mod app;
mod args;
mod cli;
mod executor;
mod display_settings;
mod drawing;
mod line;
mod line_type;
mod mission;
mod report;
mod scroll;
mod state;
mod stats;
mod tty;
mod wrap;

pub use {
    app::*,
    args::*,
    cli::*,
    display_settings::*,
    executor::*,
    drawing::*,
    line::*,
    line_type::*,
    mission::*,
    report::*,
    scroll::*,
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
    std::{env, fs::File, process::Command, str::FromStr},
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

fn main() -> Result<()> {
    configure_log();
    cli::run()?;
    debug!("bye");
    Ok(())
}
