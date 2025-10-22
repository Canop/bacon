use {
    crate::*,
    schemars::JsonSchema,
    serde::Deserialize,
    std::path::PathBuf,
};

/// A generic configuration for all exports, whatever the exporter.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ExportConfig {
    pub exporter: Option<Exporter>,
    #[serde(alias = "enabled")]
    pub auto: Option<bool>,
    pub path: Option<PathBuf>,
    pub line_format: Option<String>,
}
