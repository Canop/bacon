use {
    crate::*,
    anyhow::*,
    lazy_regex::regex_is_match,
    serde::Deserialize,
    std::{collections::HashMap, fs, path::Path},
};

/// the configuration item which may be stored as `bacon.toml`
/// along a `Cargo.toml` file
#[derive(Debug, Clone, Deserialize)]
pub struct PackageConfig {
    pub default_job: String,
    pub jobs: HashMap<String, Job>,
}

impl PackageConfig {
    pub fn get_job(&self, name: Option<&str>) -> Result<(&String, &Job)> {
        let key = name.unwrap_or(&self.default_job);
        self.jobs
            .get_key_value(key)
            .ok_or_else(|| anyhow!("Invalid bacon.toml : job not found: {:?}", key))
    }
    pub fn from_path(path: &Path) -> Result<Self> {
        let conf = toml::from_str::<PackageConfig>(&fs::read_to_string(path)?)
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
        if !conf.jobs.contains_key(&conf.default_job) {
            bail!(
                "Invalid bacon.toml : default job not found in jobs"
            );
        }
        Ok(conf)
    }
}

impl Default for PackageConfig {
    fn default() -> Self {
        toml::from_str(DEFAULT_PACKAGE_CONFIG).unwrap()
    }
}
