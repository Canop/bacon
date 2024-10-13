use {
    crate::*,
    anyhow::*,
    std::{
        collections::HashMap,
        time::Duration,
    },
};

/// The settings used in the application.
///
/// They're made from default, overriden (in order)
/// by the general prefs (global prefs.toml file), by
/// the package config (bacon.toml file in the project
/// directory) and by the launch arguments.
///
/// They're immutable during the execution of the missions.
#[derive(Debug, Clone)]
pub struct Settings {
    pub arg_job: Option<ConcreteJobRef>,
    pub additional_job_args: Vec<String>,
    pub additional_alias_args: Option<Vec<String>>,
    pub summary: bool,
    pub wrap: bool,
    pub reverse: bool,
    pub help_line: bool,
    pub no_default_features: bool,
    pub all_features: bool,
    pub features: Option<String>, // comma separated list
    pub keybindings: KeyBindings,
    pub jobs: HashMap<String, Job>,
    pub default_job: ConcreteJobRef,
    pub exports: ExportsSettings,
    pub show_changes_count: bool,
    pub on_change_strategy: Option<OnChangeStrategy>,
    pub ignored_lines: Option<Vec<LinePattern>>,
    pub grace_period: Period,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            arg_job: Default::default(),
            additional_job_args: Default::default(),
            additional_alias_args: Default::default(),
            summary: false,
            wrap: true,
            reverse: false,
            help_line: true,
            no_default_features: Default::default(),
            all_features: Default::default(),
            features: Default::default(),
            keybindings: Default::default(),
            jobs: Default::default(),
            default_job: Default::default(),
            exports: Default::default(),
            show_changes_count: false,
            on_change_strategy: None,
            ignored_lines: Default::default(),
            grace_period: Duration::from_millis(5).into(),
        }
    }
}

impl Settings {
    /// Apply one of the configuration element, overriding
    /// defaults and previously applied configuration elements
    pub fn apply_config(
        &mut self,
        config: &Config,
    ) {
        if let Some(b) = config.summary {
            self.summary = b;
        }
        if let Some(b) = config.wrap {
            self.wrap = b;
        }
        if let Some(b) = config.reverse {
            self.reverse = b;
        }
        if let Some(b) = config.help_line {
            self.help_line = b;
        }
        #[allow(deprecated)] // for compatibility
        if config.vim_keys == Some(true) {
            self.keybindings.add_vim_keys();
        }
        if let Some(keybindings) = config.keybindings.as_ref() {
            self.keybindings.add_all(keybindings);
        }
        if config.additional_alias_args.is_some() {
            self.additional_alias_args
                .clone_from(&config.additional_alias_args);
        }
        for (name, job) in &config.jobs {
            self.jobs.insert(name.clone(), job.clone());
        }
        if let Some(default_job) = &config.default_job {
            self.default_job = default_job.clone();
        }
        self.exports.apply_config(config);
        if let Some(b) = config.show_changes_count {
            self.show_changes_count = b;
        }
        if let Some(b) = config.on_change_strategy {
            self.on_change_strategy = Some(b);
        }
        if let Some(b) = config.ignored_lines.as_ref() {
            self.ignored_lines = Some(b.clone());
        }
        if let Some(p) = config.grace_period {
            self.grace_period = p;
        }
    }
    pub fn apply_args(
        &mut self,
        args: &Args,
    ) {
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
        if args.help_line {
            self.help_line = true;
        }
        if args.no_help_line {
            self.help_line = false;
        }
        if args.export_locations {
            self.exports.set_locations_export_auto(true);
        }
        if args.no_export_locations {
            self.exports.set_locations_export_auto(false);
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
            self.features.clone_from(&args.features);
        }
        self.additional_job_args
            .clone_from(&args.additional_job_args);
    }
    pub fn check(&self) -> Result<()> {
        if self.jobs.is_empty() {
            bail!("Invalid configuration : no job found");
        }
        if let NameOrAlias::Name(name) = &self.default_job.name_or_alias {
            if !self.jobs.contains_key(name) {
                bail!("Invalid configuration : default job ({name:?}) not found in jobs");
            }
        }
        Ok(())
    }
}
