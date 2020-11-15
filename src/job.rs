use {
    serde::Deserialize,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Job {

    /// guaranteed not empty by the PackageConfig::from_path
    /// loader
    pub command: Vec<String>,

    /// whether we need to capture stdout too (stderr is
    /// always captured)
    #[serde(default)]
    pub need_stdout: bool,
}
