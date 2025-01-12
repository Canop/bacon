use {
    crate::*,
    serde::Deserialize,
    std::collections::HashMap,
};

/// One of the possible job that bacon can run
#[derive(Debug, Clone, Deserialize)]
pub struct Job {
    /// List of alternative names to run this job.
    pub alias: Option<Vec<String>>,

    /// Whether to consider that we can have a success
    /// when we have test failures
    #[serde(default)]
    pub allow_failures: bool,

    /// Whether to consider that we can have a success
    /// when we have warnings. This is especially useful
    /// for "cargo run" jobs
    #[serde(default)]
    pub allow_warnings: bool,

    /// The analyzer interpreting the output of the command, the
    /// standard cargo dedicated one if not provided
    pub analyzer: Option<AnalyzerRef>,

    /// Whether gitignore rules must be applied
    pub apply_gitignore: Option<bool>,

    /// Whether to wait for the computation to finish before
    /// to display it on screen
    ///
    /// This is true by default. Set it to false if you want
    /// the previous computation result to be replaced with
    /// the new one as soon as it starts.
    #[serde(default = "default_true")]
    pub background: bool,

    /// The tokens making the command to execute (first one
    /// is the executable).
    /// This vector is guaranteed not empty
    /// by the PackageConfig::from_path loader
    pub command: Vec<String>,

    /// Whether to apply the default watch list, which is
    /// `["src", "tests", "benches", "examples", "build.rs"]`
    ///
    /// This is true by default. Set it to false if you want
    /// to watch nothing, or only the directories you set in
    /// `watch`.
    pub default_watch: Option<bool>,

    /// Env vars to set for this job execution
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether to expand environment variables in the command
    #[serde(default = "default_true")]
    pub expand_env_vars: bool,

    /// Whether to insert extraneous arguments provided by bacon or end users
    ///
    /// Eg: --all-features or anything after -- in bacon incantation
    #[serde(default = "default_true")]
    pub extraneous_args: bool,

    /// A lit of glob patterns to ignore
    #[serde(default)]
    pub ignore: Vec<String>,

    /// Patterns of lines which should be ignored. Patterns of
    /// the prefs or bacon.toml can be overridden at the job
    pub ignored_lines: Option<Vec<LinePattern>>,

    /// A kill command. If not provided, SIGKILL is used.
    pub kill: Option<Vec<String>>,

    /// Whether we need to capture stdout too (stderr is
    /// always captured)
    #[serde(default)]
    pub need_stdout: bool,

    /// How to handle changes: either immediately kill the current job
    /// then restart it, or wait for the current job to finish before
    /// restarting it.
    pub on_change_strategy: Option<OnChangeStrategy>,

    /// The optional action to run when there's no
    /// error, warning or test failures
    #[serde(default)]
    pub on_success: Option<Action>,

    /// A list of directories that will be watched if the job
    /// is run on a package.
    /// src, examples, tests, and benches are implicitly included
    /// unless you `set default_watch` to false.
    pub watch: Option<Vec<String>>,
}

static DEFAULT_ARGS: &[&str] = &["--color", "always"];

// waiting for https://github.com/serde-rs/serde/issues/368
fn default_true() -> bool {
    true
}

impl Job {
    /// Build a `Job` for a cargo alias
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
            alias: None,
            kill: None,
            default_watch: None,
            expand_env_vars: true,
            watch: None,
            need_stdout: false,
            on_success: None,
            allow_warnings: false,
            allow_failures: false,
            apply_gitignore: None,
            env: Default::default(),
            background: true,
            extraneous_args: true,
            on_change_strategy: None,
            analyzer: Some(AnalyzerRef::Standard),
            ignored_lines: None,
            ignore: Default::default(),
        }
    }
}
