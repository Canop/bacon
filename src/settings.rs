use {crate::*, anyhow::*, std::collections::HashMap};

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
    pub arg_job: Option<ConcreteJobRef>,
    pub additional_job_args: Vec<String>,
    pub additional_alias_args: Option<Vec<String>>,
    pub summary: bool,
    pub wrap: bool,
    pub reverse: bool,
    pub no_default_features: bool,
    pub all_features: bool,
    pub features: Option<String>, // comma separated list
    pub keybindings: KeyBindings,
    pub export_locations: bool,
    pub jobs: HashMap<String, Job>,
    pub default_job: ConcreteJobRef,
}

impl Settings {
    pub fn apply_config(&mut self, config: &Config) {
        if let Some(b) = config.summary {
            self.summary = b;
        }
        if let Some(b) = config.wrap {
            self.wrap = b;
        }
        if let Some(b) = config.reverse {
            self.reverse = b;
        }
        if let Some(b) = config.export_locations {
            self.export_locations = b;
        }
        if config.vim_keys == Some(true) {
            self.keybindings.add_vim_keys();
        }
        if let Some(keybindings) = config.keybindings.as_ref() {
            self.keybindings.add_all(keybindings);
        }
        if config.additional_alias_args.is_some() {
            self.additional_alias_args = config.additional_alias_args.clone();
        }
        for (name, job) in &config.jobs {
            self.jobs.insert(name.clone(), job.clone());
        }
        if let Some(default_job) = &config.default_job {
            self.default_job = default_job.clone();
        }
    }
    pub fn apply_args(&mut self, args: &Args) {
        if let Some(job) = &args.job {
            self.arg_job = Some(job.clone());
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
    pub fn check(&self) -> Result<()> {
        if self.jobs.is_empty() {
            bail!("Invalid configuration : no job found");
        }
        if let ConcreteJobRef::Name(name) = &self.default_job {
            if !self.jobs.contains_key(name) {
                bail!("Invalid configuration : default job ({name:?}) not found in jobs");
            }
        }
        Ok(())
    }
}
