use serde::Deserialize;

/// One of the possible job that bacon can run
#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    /// The tokens making the command to execute (first one
    /// is the executable).
    /// This vector is guaranteed not empty
    /// by the PackageConfig::from_path loader
    pub command: Vec<String>,

    /// A list of directories that will be watched if the job
    /// is run on a package.
    /// src is implicitly included.
    #[serde(default)]
    pub watch: Vec<String>,

    /// whether we need to capture stdout too (stderr is
    /// always captured)
    #[serde(default)]
    pub need_stdout: bool,
}
