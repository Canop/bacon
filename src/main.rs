#![ allow( dead_code, unused_imports ) ]

mod app;
mod cli;
mod computer;
mod item;
mod report;
mod state;
mod watcher;

pub use {
    app::*,
    cli::*,
    computer::*,
    item::*,
    report::*,
    state::*,
    watcher::*,
};

#[macro_use]
extern crate log;

use {
    anyhow::*,
    log::LevelFilter,
    simplelog,
    std::{
        env,
        fs::File,
        io::{self, Write},
        process::Command,
        str::FromStr,
    },
};


/// configure the application log according to env variable.
///
/// There's no log unless the BACON_LOG environment variable is set to
///  a valid log level (trace, debug, info, warn, error, off)
/// Example:
///      BACON_LOG=info broot
/// As bacon is a terminal application, we only log to a file (dev.log)
fn configure_log() {
    let level = env::var("BACON_LOG").unwrap_or_else(|_| "off".to_string());
    if level == "off" {
        return;
    }
    if let Ok(level) = LevelFilter::from_str(&level) {
        simplelog::WriteLogger::init(
            level,
            simplelog::Config::default(),
            File::create("dev.log").expect("Log file can't be created"),
        )
        .expect("log initialization failed");
        info!(
            "Starting Bacon v{} with log level {}",
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
