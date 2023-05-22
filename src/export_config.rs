use {
    serde::Deserialize,
    std::path::PathBuf,
};

/// Configuration of export, may be in prefs.toml
/// or bacon.toml
#[derive(Debug, Clone, Deserialize)]
pub struct ExportConfig {
    pub enabled: Option<bool>,
    pub path: Option<PathBuf>,
    pub line_format: Option<String>,
}
