use {
    serde::Deserialize,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Job {

    /// guaranteed not empty by the PackageConfig::from_path
    /// loader
    pub command: Vec<String>,
}
