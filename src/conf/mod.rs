mod action;
mod auto_refresh;
mod cargo_wrapped_config;
mod config;
mod defaults;
mod keybindings;
mod settings;
mod skin;
mod skin_color;

pub use {
    action::*,
    auto_refresh::*,
    cargo_wrapped_config::*,
    config::*,
    defaults::*,
    keybindings::*,
    settings::*,
    skin::*,
    skin_color::*,
};

use std::{
    borrow::Cow,
    path::{
        Path,
        PathBuf,
    },
};

/// Expand the tilde and make the path absolute, a relative path
/// being taken from the given directory
pub fn resolve_path(
    path: &Path,
    base_dir: &Path,
) -> PathBuf {
    let expanded = expand_tilde(path);
    if expanded.is_absolute() {
        expanded.into_owned()
    } else {
        base_dir.join(expanded)
    }
}

/// Replace a leading `~` with the path of the user's home directory
pub fn expand_tilde(path: &Path) -> Cow<'_, Path> {
    let Ok(rest) = path.strip_prefix("~") else {
        return Cow::Borrowed(path);
    };
    match directories_next::UserDirs::new() {
        Some(user_dirs) => Cow::Owned(user_dirs.home_dir().join(rest)),
        None => Cow::Borrowed(path),
    }
}

/// If the system can manage application preferences, return the
/// canonical path to the bacon preferences file
pub fn bacon_prefs_path() -> Option<std::path::PathBuf> {
    directories_next::ProjectDirs::from("org", "dystroy", "bacon")
        .map(|project_dir| project_dir.config_dir().join("prefs.toml"))
}

/// Return the path given by the env var, if it exists (doesn't check whether
/// it's a correct configuration file)
pub fn config_path_from_env(env_var_name: &str) -> Option<PathBuf> {
    let path = std::env::var_os(env_var_name)?;
    let path = Path::new(&path);
    if path.exists() {
        Some(path.to_path_buf())
    } else {
        // some users may want to use an env var to point to a file that may not always exist
        // so we don't throw an error here
        warn!("Env var {env_var_name:?} points to file {path:?} which does not exist");
        None
    }
}
