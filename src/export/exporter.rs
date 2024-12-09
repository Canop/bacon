use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exporter {
    /// The analyzer is tasked with doing an export while analyzing the
    /// command output
    #[serde(alias = "analyzer")]
    Analyser,
    /// This exporter doesn't exist at the moment
    #[serde(alias = "analyzis")]
    Analysis,
    JsonReport,
    Locations,
}
