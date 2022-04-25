use {
    crate::*,
    serde::Deserialize,
};

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

    /// the optional action to run when there's no
    /// error, warning or test failures
    #[serde(default)]
    pub on_success: Option<Action>,
}

impl Job {
    /// Builds a `Job` from a cargo alias. The user is expected to know what
    /// they are doing here.
    pub fn from_alias(alias_name: &str, settings: &Settings) -> Self {
        let mut command = vec!["cargo".to_string(), alias_name.to_string()];

        if !settings.no_default_alias_args {
            command.extend_from_slice(&["--color".to_string(), "always".to_string()]);
        }

        Self {
            command,
            watch: Vec::new(),
            need_stdout: false,
            on_success: None,
        }
    }
}
