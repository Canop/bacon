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
    pub reverse: Option<bool>,
    pub vim_keys: Option<bool>,
}

impl Prefs {
    pub fn from_path(path: &Path) -> Result<Self> {
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }
}

impl Default for Prefs {
    fn default() -> Self {
        toml::from_str(DEFAULT_PREFS).unwrap()
    }
}


