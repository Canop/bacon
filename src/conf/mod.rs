mod action;
mod args;
mod config;
mod defaults;
mod keybindings;

mod settings;

pub use {
    action::*,
    args::*,
    config::*,
    defaults::*,
    keybindings::*,
    settings::*,
};

use std::path::{
    Path,
    PathBuf,
};

/// If the system can manage application preferences, return the
/// canonical path to the bacon preferences file
pub fn bacon_prefs_path() -> Option<std::path::PathBuf> {
    directories_next::ProjectDirs::from("org", "dystroy", "bacon")
        .map(|project_dir| project_dir.config_dir().join("prefs.toml"))
}

pub fn config_path_from_env(env_var_name: &str) -> Option<PathBuf> {
    let path = std::env::var_os(env_var_name)?;
    let path = Path::new(&path);
    if path.exists() {
        Some(path.to_path_buf())
    } else {
        // some users may want to use an env var to point to a file that may not always exist
        // so we don't throw an error here
        warn!(
            "Env var {:?} points to file {:?} which does not exist",
            env_var_name, path
        );
        None
    }
}
