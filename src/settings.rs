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
    pub arg_job_name: Option<JobType>,
    pub additional_job_args: Vec<String>,
    pub summary: bool,
    pub wrap: bool,
    pub reverse: bool,
    pub no_default_alias_args: bool,
    pub no_default_features: bool,
    pub all_features: bool,
    pub features: Option<String>, // comma separated list
    pub keybindings: KeyBindings,
    pub export_locations: bool,
}

/// Remember whether the user gave us a job or an alias.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JobType {
    Job(String),
    Alias(String),
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
        if let Some(b) = prefs.export_locations {
            self.export_locations = b;
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
        if let Some(alias_name) = &args.alias {
            self.arg_job_name = Some(JobType::Alias(alias_name.clone()));
        } else if let Some(job_name) = &args.job {
            self.arg_job_name = Some(JobType::Job(job_name.clone()));
        }
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
        if args.export_locations {
            self.export_locations = true;
        }
        if args.no_export_locations {
            self.export_locations = false;
        }
        if args.reverse {
            self.reverse = true;
        }
        if args.no_default_alias_args {
            self.no_default_alias_args = true;
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
        self.additional_job_args = args.additional_job_args.clone();
    }
}

impl JobType {
    pub fn name(&self) -> &str {
        match self {
            JobType::Job(s) => s,
            JobType::Alias(s) => s,
        }
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Job(j) => f.write_str(j),
            Self::Alias(a) => f.write_str(a),
        }
    }
}
