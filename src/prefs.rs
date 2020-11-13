use {
    anyhow::*,
    crate::*,
    serde::Deserialize,
    std::{
        fs,
        path::Path,
    },
    toml,
};

/// the configuration item which may be stored as `prefs.toml`
/// in the xdg config directory
#[derive(Debug, Clone, Deserialize)]
pub struct Prefs {
    pub summary: Option<bool>,
    pub wrap: Option<bool>,
}

impl Prefs {
    pub fn from_path(path: &Path) -> Result<Self> {
        let prefs = toml::from_str::<Prefs>(&fs::read_to_string(path)?)?;
        Ok(prefs)
    }
}

impl Default for Prefs {
    fn default() -> Self {
        toml::from_str(DEFAULT_PREFS).unwrap()
    }
}


