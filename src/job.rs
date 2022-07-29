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

    /// whether to consider that we have a success when
    /// we only have warnings. This is especially useful
    /// for "cargo run" jobs
    #[serde(default)]
    pub allow_warnings: bool,
}

static DEFAULT_ARGS: &[&str] = &["--color", "always"];

impl Job {
    /// Builds a `Job` for a cargo alias
    pub fn from_alias(
        alias_name: &str,
        settings: &Settings,
    ) -> Self {
        let mut command = vec!["cargo".to_string(), alias_name.to_string()];
        if let Some(additional_args) = settings.additional_alias_args.as_ref() {
            for arg in additional_args {
                command.push(arg.to_string())
            }
        } else {
            for arg in DEFAULT_ARGS {
                command.push(arg.to_string())
            }
        }
        Self {
            command,
            watch: Vec::new(),
            need_stdout: false,
            on_success: None,
            allow_warnings: false,
        }
    }
}
