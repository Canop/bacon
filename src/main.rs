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

/// Knowledge is power
fn main() -> anyhow::Result<()> {
    cli_log::init("bacon");
    cli::run()?;
    info!("bye");
    Ok(())
}
