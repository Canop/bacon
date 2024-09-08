mod action;
mod app;
mod args;
mod auto_refresh;
mod cli;
mod command_output;
mod command_result;
mod config;
mod defaults;
mod drawing;
mod examples;
mod executor;
mod export_config;
mod export_settings;
mod failure;
mod help_line;
mod help_page;
mod ignorer;
mod internal;
mod job;
mod job_ref;
mod job_stack;
mod keybindings;
mod line;
mod line_analysis;
mod line_type;
mod list_jobs;
mod mission;
mod mission_location;
mod on_change_strategy;
mod report;
mod scroll;
mod settings;
mod state;
mod stats;
mod tty;
mod wrap;
mod wrapped_command_output;
mod wrapped_report;

pub use {
    action::*,
    args::*,
    auto_refresh::*,
    cli::*,
    command_output::*,
    command_result::*,
    config::*,
    defaults::*,
    drawing::*,
    examples::*,
    executor::*,
    export_config::*,
    export_settings::*,
    failure::*,
    help_line::*,
    help_page::*,
    ignorer::*,
    internal::*,
    job::*,
    job_ref::*,
    job_stack::*,
    keybindings::*,
    line::*,
    line_analysis::*,
    line_type::*,
    list_jobs::*,
    mission::*,
    mission_location::*,
    on_change_strategy::*,
    report::*,
    scroll::*,
    settings::*,
    state::*,
    stats::*,
    tty::*,
    wrap::*,
    wrapped_command_output::*,
    wrapped_report::*,
};

#[macro_use]
extern crate cli_log;

/// Knowledge is power
fn main() -> anyhow::Result<()> {
    init_cli_log!();
    cli::run()?;
    info!("bye");
    Ok(())
}
