use crate::*;

/// The settings used in the application.
///
/// They're made from default, overriden (in order)
/// by the general prefs (global prefs.toml file), by
/// the package config (bacon.toml file in the project
/// directory) and by the launch arguments.
///
/// They're immutable during the execution of the missions.
#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub summary: bool,
    pub wrap: bool,
    pub reverse: bool,
    pub no_default_features: bool,
    pub all_features: bool,
    pub features: Option<String>, // comma separated list
    pub keybindings: KeyBindings,
}

impl Settings {
    pub fn apply_prefs(&mut self, prefs: &Prefs) {
        if let Some(b) = prefs.summary {
            self.summary = b;
        }
        if let Some(b) = prefs.wrap {
            self.wrap = b;
        }
        if let Some(b) = prefs.reverse {
            self.reverse = b;
        }
        if prefs.vim_keys == Some(true) {
            self.keybindings.add_vim_keys();
        }
        if let Some(pref_keybindings) = prefs.keybindings.as_ref() {
            self.keybindings.add_all(pref_keybindings);
        }
    }
    pub fn apply_package_config(&mut self, package_config: &PackageConfig) {
        if let Some(keybindings) = package_config.keybindings.as_ref() {
            self.keybindings.add_all(keybindings);
        }
    }
    pub fn apply_args(&mut self, args: &Args) {
        if args.no_summary {
            self.summary = false;
        }
        if args.summary {
            self.summary = true;
        }
        if args.no_wrap {
            self.wrap = false;
        }
        if args.wrap {
            self.wrap = true;
        }
        if args.no_reverse {
            self.reverse = false;
        }
        if args.reverse {
            self.reverse = true;
        }
        if args.no_default_features {
            self.no_default_features = true;
        }
        if args.all_features {
            self.all_features = true;
        }
        if args.features.is_some() {
            self.features = args.features.clone();
        }
    }
}
