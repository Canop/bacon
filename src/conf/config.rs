use {
    crate::*,
    anyhow::*,
    lazy_regex::regex_is_match,
    serde::Deserialize,
    std::{
        collections::HashMap,
        fs,
        path::Path,
    },
};

/// A configuration item which may be stored in various places, eg as `bacon.toml`
/// along a `Cargo.toml` file or as `prefs.toml` in the xdg config directory.
///
/// Leaf values are options (and not Default) so that they don't
/// override previously set values when applied to settings.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub additional_alias_args: Option<Vec<String>>,

    pub default_job: Option<ConcreteJobRef>,

    /// Whether to apply the default watch list, which is
    /// `["src", "tests", "benches", "examples", "build.rs"]`
    ///
    /// This is true by default. Set it to false if you want
    /// to watch nothing, or only the directories you set in
    /// `watch`.
    pub default_watch: Option<bool>,

    /// locations export
    #[deprecated(since = "2.22.0", note = "use exports.locations")]
    pub export: Option<ExportConfig>,

    #[deprecated(since = "2.9.0", note = "use exports.locations.auto")]
    pub export_locations: Option<bool>,

    #[serde(default)]
    pub exports: HashMap<String, ExportConfig>,

    /// The delay between a file event and the real start of the
    /// task. Other file events occuring during this period will be
    /// ignored.
    pub grace_period: Option<Period>,

    pub help_line: Option<bool>,

    /// A lit of glob patterns to ignore
    #[serde(default)]
    pub ignore: Vec<String>,

    /// Patterns of lines which should be ignored. Patterns of
    /// the prefs or bacon.toml can be overridden at the job
    pub ignored_lines: Option<Vec<LinePattern>>,

    #[serde(default)]
    pub jobs: HashMap<String, Job>,

    pub keybindings: Option<KeyBindings>,

    pub on_change_strategy: Option<OnChangeStrategy>,

    pub reverse: Option<bool>,

    pub show_changes_count: Option<bool>,

    pub summary: Option<bool>,

    #[deprecated(since = "2.0.0", note = "use keybindings")]
    pub vim_keys: Option<bool>,

    /// A list of files and directories that will be watched if the job
    /// is run on a package.
    ///
    /// src, examples, tests, benches, and build.rs are implicitly
    /// included unless you `set default_watch` to false.
    pub watch: Option<Vec<String>>,

    pub wrap: Option<bool>,

    /// Env vars to set for all job executions
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether to beep when the job ends
    pub beep_on_end: Option<bool>,
}

impl Config {
    /// Load from zero to two configuration items from the provided path which
    /// must be in TOML format but may not exist.
    ///
    /// Expected structures are either bacon config or a cargo.toml file (which
    /// may contain a workspace.metadata.bacon key and a package.metadata.bacon key).
    pub fn from_path_detect(path: &Path) -> Result<Vec<Self>> {
        if !path.exists() {
            return Ok(Vec::default());
        }
        let file_name = path.file_name().and_then(|f| f.to_str());
        if file_name == Some("Cargo.toml") {
            load_config_from_cargo_toml(path)
        } else {
            Ok(vec![Self::from_path(path)?])
        }
    }
    /// Load a configuration item filling the provided path in TOML
    pub fn from_path(path: &Path) -> Result<Self> {
        let conf = toml::from_str::<Self>(&fs::read_to_string(path)?)
            .with_context(|| format!("Failed to parse configuration file at {:?}", path))?;
        for (name, job) in &conf.jobs {
            if !regex_is_match!(r#"^[\w-]+$"#, name) {
                bail!("Invalid configuration : Illegal job name : {:?}", name);
            }
            if job.command.is_empty() {
                bail!("Invalid configuration : empty command for job {:?}", name);
            }
        }
        Ok(conf)
    }
    pub fn default_package_config() -> Self {
        toml::from_str(DEFAULT_PACKAGE_CONFIG).unwrap()
    }
    pub fn default_prefs() -> Self {
        toml::from_str(DEFAULT_PREFS).unwrap()
    }
}

#[test]
fn test_default_files() {
    let mut settings = Settings::default();
    settings.apply_config(&Config::default_prefs());
    settings.apply_config(&Config::default_package_config());
    settings.check().unwrap();
}
