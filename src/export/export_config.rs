use {
    crate::*,
    schemars::JsonSchema,
    serde::Deserialize,
    std::path::PathBuf,
};

/// A generic configuration for all exports, whatever the exporter.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ExportConfig {
    /// Exporter backend that should produce the output.
    pub exporter: Option<Exporter>,

    /// Whether the export should run automatically after each mission.
    #[serde(alias = "enabled")]
    pub auto: Option<bool>,

    /// Destination path where the exporter writes its output.
    pub path: Option<PathBuf>,

    /// Optional format string used by exporters that write line-based data.
    pub line_format: Option<String>,
}
