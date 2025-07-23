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

    /// Default config for a job
    #[serde(flatten)]
    pub all_jobs: Job,

    /// locations export
    #[deprecated(since = "2.22.0", note = "use exports.locations")]
    pub export: Option<ExportConfig>,

    #[deprecated(since = "2.9.0", note = "use exports.locations.auto")]
    pub export_locations: Option<bool>,

    #[serde(default)]
    pub exports: HashMap<String, ExportConfig>,

    pub help_line: Option<bool>,

    #[serde(default)]
    pub jobs: HashMap<String, Job>,

    pub keybindings: Option<KeyBindings>,

    pub reverse: Option<bool>,

    pub summary: Option<bool>,

    #[deprecated(since = "2.0.0", note = "use keybindings")]
    pub vim_keys: Option<bool>,

    /// Whether to listen for actions on a unix socket (if on unix)
    pub listen: Option<bool>,

    pub wrap: Option<bool>,
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
            .with_context(|| format!("Failed to parse configuration file at {path:?}"))?;
        for (name, job) in &conf.jobs {
            if !regex_is_match!(r#"^[\w-]+$"#, name) {
                bail!("Invalid configuration : Illegal job name : {name:?}");
            }
            if job.command.is_empty() {
                bail!("Invalid configuration : empty command for job {name:?}");
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
