use {
    crate::*,
    anyhow::*,
    serde::Deserialize,
    std::{fs, path::Path},
};

/// the configuration item which may be stored as `prefs.toml`
/// in the xdg config directory
#[derive(Debug, Clone, Deserialize)]
pub struct Prefs {
    pub summary: Option<bool>,
    pub wrap: Option<bool>,
    pub reverse: Option<bool>,
    pub vim_keys: Option<bool>, // deprecated thanks to keybindings
    pub export_locations: Option<bool>,
    pub keybindings: Option<KeyBindings>,
    pub additional_alias_args: Option<Vec<String>>,
}

impl Prefs {
    pub fn from_path(path: &Path) -> Result<Self> {
        let prefs = toml::from_str(&fs::read_to_string(path)?)
            .with_context(|| format!("Failed to parse preference file at {:?}", path))?;
        Ok(prefs)
    }
}

impl Default for Prefs {
    fn default() -> Self {
        toml::from_str(DEFAULT_PREFS).unwrap()
    }
}
