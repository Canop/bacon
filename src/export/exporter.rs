use {
    schemars::JsonSchema,
    serde::Deserialize,
};

/// Export backend available for Bacon exports.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Exporter {
    /// The analyzer is tasked with doing an export while analyzing the
    /// command output
    #[serde(alias = "analyzer")]
    Analyser,
    /// This exporter doesn't exist at the moment
    #[serde(alias = "analyzis")]
    Analysis,
    /// Emit a machine-readable JSON report for the mission.
    JsonReport,
    /// Produce a list of file locations for editors or other tools.
    Locations,
}
