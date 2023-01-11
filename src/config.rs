use {
    crate::*,
    anyhow::*,
    lazy_regex::regex_is_match,
    serde::Deserialize,
    std::{collections::HashMap, fs, path::Path},
};

/// The configuration item which may be stored either as `bacon.toml`
/// along a `Cargo.toml` file or as `prefs.toml` in the xdg config directory
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub summary: Option<bool>,
    pub wrap: Option<bool>,
    pub reverse: Option<bool>,
    pub vim_keys: Option<bool>, // deprecated thanks to keybindings
    pub export_locations: Option<bool>,
    pub keybindings: Option<KeyBindings>,
    pub additional_alias_args: Option<Vec<String>>,
    #[serde(default)]
    pub jobs: HashMap<String, Job>,
    pub default_job: Option<ConcreteJobRef>,
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self> {
        let conf = toml::from_str::<Self>(&fs::read_to_string(path)?)
            .with_context(|| format!("Failed to parse configuration file at {:?}", path))?;
        if conf.jobs.is_empty() {
            bail!("Invalid bacon.toml : no job found");
        }
        for (name, job) in &conf.jobs {
            if !regex_is_match!(r#"^[\w-]+$"#, name) {
                bail!(
                    "Invalid bacon.toml : Illegal job name : {:?}",
                    name
                );
            }
            if job.command.is_empty() {
                bail!(
                    "Invalid bacon.toml : empty command for job {:?}",
                    name
                );
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
