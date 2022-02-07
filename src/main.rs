mod action;
mod app;
mod args;
mod cli;
mod command_output;
mod command_result;
mod defaults;
mod drawing;
mod executor;
mod failure;
mod help_line;
mod internal;
mod job;
mod keybindings;
mod line;
mod line_analysis;
mod line_type;
mod list_jobs;
mod mission;
mod mission_location;
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
    action::*,
    app::*,
    args::*,
    cli::*,
    command_output::*,
    command_result::*,
    defaults::*,
    drawing::*,
    executor::*,
    failure::*,
    help_line::*,
    internal::*,
    job::*,
    keybindings::*,
    line::*,
    line_analysis::*,
    line_type::*,
    list_jobs::*,
    mission::*,
    mission_location::*,
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

#[macro_use] extern crate cli_log;

/// Knowledge is power
fn main() -> anyhow::Result<()> {
    init_cli_log!();
    cli::run()?;
    info!("bye");
    Ok(())
}
