use {
    crate::*,
    serde::Deserialize,
    std::{
        collections::HashMap,
        fs,
        path::PathBuf,
    },
};

/// One of the possible jobs that bacon can run
#[derive(Debug, Default, Clone, Deserialize, PartialEq)]
pub struct Job {
    /// Whether to consider that we can have a success
    /// when we have test failures
    pub allow_failures: Option<bool>,

    /// Whether to consider that we can have a success
    /// when we have warnings. This is especially useful
    /// for "cargo run" jobs
    pub allow_warnings: Option<bool>,

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
    pub background: Option<bool>,

    /// The tokens making the command to execute (first one
    /// is the executable).
    #[serde(default)]
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

    /// Path to a file containing environment variables to load
    /// The file should contain KEY=VALUE pairs, one per line
    pub env_file: Option<PathBuf>,

    /// Whether to expand environment variables in the command
    pub expand_env_vars: Option<bool>,

    /// Whether to insert extraneous arguments provided by bacon or end users
    ///
    /// Eg: --all-features or anything after -- in bacon incantation
    pub extraneous_args: Option<bool>,

    /// A list of glob patterns to ignore
    #[serde(default)]
    pub ignore: Vec<String>,

    /// Patterns of lines which should be ignored. Patterns of
    /// the prefs or bacon.toml can be overridden at the job
    pub ignored_lines: Option<Vec<LinePattern>>,

    /// A kill command. If not provided, SIGKILL is used.
    pub kill: Option<Vec<String>>,

    /// Whether we need to capture stdout too (stderr is
    /// always captured)
    pub need_stdout: Option<bool>,

    /// How to handle changes: either immediately kill the current job
    /// then restart it, or wait for the current job to finish before
    /// restarting it.
    pub on_change_strategy: Option<OnChangeStrategy>,

    /// The optional action to run when there's no
    /// error, warning or test failures
    /// (depending on whether allow_warnings is true or false)
    ///
    /// Could be made a vec in the future but that would involve
    /// explaining subtleties like the fact that those actions stop
    /// after the first one ending the mission or doing a refresh
    #[serde(default)]
    pub on_success: Option<Action>,

    pub grace_period: Option<Period>,

    /// The optional action to run when it's not a success
    #[serde(default)]
    pub on_failure: Option<Action>,

    /// A list of directories that will be watched if the job
    /// is run on a package.
    /// src, examples, tests, and benches are implicitly included
    /// unless you `set default_watch` to false.
    pub watch: Option<Vec<String>>,

    pub show_changes_count: Option<bool>,

    #[serde(default)]
    pub sound: SoundConfig,

    /// An optional working directory for the job command, which
    /// would override the package directory.
    pub workdir: Option<PathBuf>,

    #[serde(default)]
    pub skin: BaconSkin,
}

static DEFAULT_ARGS: &[&str] = &["--color", "always"];

impl Job {
    /// Load environment variables from a file
    /// Expected format: KEY=VALUE, one per line
    /// Lines starting with # are treated as comments and ignored
    /// Empty lines are ignored
    /// If env_file is relative, it's resolved relative to base_dir (usually package directory)
    pub fn load_env_from_file(env_file: &PathBuf, base_dir: Option<&PathBuf>) -> Result<HashMap<String, String>, std::io::Error> {
        let resolved_path = Self::filepath_of(env_file, base_dir);

        let content = fs::read_to_string(&resolved_path)?;
        let mut env_vars = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            Self::parse_key_value_pairs(&mut env_vars, line);
        }

        Ok(env_vars)
    }

    fn filepath_of(env_file: &PathBuf, base_dir: Option<&PathBuf>) -> PathBuf {
        if env_file.is_absolute() {
            env_file.clone()
        } else if let Some(base) = base_dir {
            base.join(env_file)
        } else {
            env_file.clone()
        }
    }

    fn parse_key_value_pairs(env_vars: &mut HashMap<String, String>, line: &str) {
        if let Some(eq_pos) = line.find('=') {
            let key = line[..eq_pos].trim().to_string();
            let value = line[eq_pos + 1..].trim().to_string();

            let value = if Self::has_surrounding_quotes(&value) {
                Self::remove_surrounding_quotes(&value)
            } else {
                value
            };

            env_vars.insert(key, value);
        }
    }

    fn remove_surrounding_quotes(value: &str) -> String {
        value[1..value.len() - 1].to_string()
    }

    fn has_surrounding_quotes(value: &str) -> bool {
        (value.starts_with('"') && value.ends_with('"')) ||
            (value.starts_with('\'') && value.ends_with('\''))
    }

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
            ..Default::default()
        }
    }
    pub fn allow_failures(&self) -> bool {
        self.allow_failures.unwrap_or(false)
    }
    pub fn allow_warnings(&self) -> bool {
        self.allow_warnings.unwrap_or(false)
    }
    pub fn background(&self) -> bool {
        self.background.unwrap_or(true)
    }
    pub fn default_watch(&self) -> bool {
        self.default_watch.unwrap_or(true)
    }
    pub fn expand_env_vars(&self) -> bool {
        self.expand_env_vars.unwrap_or(true)
    }
    pub fn need_stdout(&self) -> bool {
        self.need_stdout.unwrap_or(false)
    }
    pub fn extraneous_args(&self) -> bool {
        self.extraneous_args.unwrap_or(true)
    }
    pub fn show_changes_count(&self) -> bool {
        self.show_changes_count.unwrap_or(false)
    }
    pub fn grace_period(&self) -> Period {
        self.grace_period
            .unwrap_or(std::time::Duration::from_millis(15).into())
    }
    pub fn on_change_strategy(&self) -> OnChangeStrategy {
        self.on_change_strategy
            .unwrap_or(OnChangeStrategy::WaitThenRestart)
    }
    pub fn apply(
        &mut self,
        job: &Job,
    ) {
        if let Some(b) = job.allow_failures {
            self.allow_failures = Some(b);
        }
        if let Some(b) = job.allow_warnings {
            self.allow_warnings = Some(b);
        }
        if let Some(v) = job.analyzer {
            self.analyzer = Some(v);
        }
        if let Some(b) = job.apply_gitignore {
            self.apply_gitignore = Some(b);
        }
        if let Some(b) = job.background {
            self.background = Some(b);
        }
        if !job.command.is_empty() {
            self.command.clone_from(&job.command);
        }
        if let Some(b) = job.default_watch {
            self.default_watch = Some(b);
        }
        for (k, v) in &job.env {
            self.env.insert(k.clone(), v.clone());
        }
        if let Some(env_file) = job.env_file.as_ref() {
            if let Ok(file_env_vars) = Self::load_env_from_file(env_file, None) {
                for (key, value) in file_env_vars {
                    // env_file variables have lower priority than direct env vars
                    self.env.entry(key).or_insert(value);
                }
            }
        }
        if let Some(env_file) = job.env_file.as_ref() {
            self.env_file = Some(env_file.clone());
        }
        if let Some(b) = job.expand_env_vars {
            self.expand_env_vars = Some(b);
        }
        if let Some(b) = job.extraneous_args {
            self.extraneous_args = Some(b);
        }
        for v in &job.ignore {
            if !self.ignore.contains(v) {
                self.ignore.push(v.clone());
            }
        }
        if let Some(v) = job.ignored_lines.as_ref() {
            self.ignored_lines = Some(v.clone());
        }
        if let Some(v) = job.kill.as_ref() {
            self.kill = Some(v.clone());
        }
        if let Some(b) = job.need_stdout {
            self.need_stdout = Some(b);
        }
        if let Some(v) = job.on_change_strategy {
            self.on_change_strategy = Some(v);
        }
        if let Some(v) = job.on_success.as_ref() {
            self.on_success = Some(v.clone());
        }
        if let Some(v) = job.grace_period {
            self.grace_period = Some(v);
        }
        if let Some(v) = job.on_failure.as_ref() {
            self.on_failure = Some(v.clone());
        }
        if let Some(v) = job.watch.as_ref() {
            self.watch = Some(v.clone());
        }
        if let Some(b) = job.show_changes_count {
            self.show_changes_count = Some(b);
        }
        self.sound.apply(&job.sound);
        if let Some(p) = job.workdir.as_ref() {
            self.workdir = Some(p.clone());
        }
        self.skin.apply(job.skin);
    }
}

#[test]
fn test_job_apply() {
    use std::str::FromStr;
    let mut base_job = Job::default();
    let job_to_apply = Job {
        allow_failures: Some(true),
        allow_warnings: Some(false),
        analyzer: Some(AnalyzerRef::Nextest),
        apply_gitignore: Some(false),
        background: Some(false),
        command: vec!["cargo".to_string(), "test".to_string()],
        default_watch: Some(false),
        env: vec![("RUST_LOG".to_string(), "debug".to_string())]
            .into_iter()
            .collect(),
        env_file: Some(PathBuf::from("/path/to/.env")),
        expand_env_vars: Some(false),
        extraneous_args: Some(false),
        ignore: vec!["special-target".to_string(), "generated".to_string()],
        ignored_lines: Some(vec![LinePattern::from_str("half-error.*").unwrap()]),
        kill: Some(vec!["die".to_string()]),
        need_stdout: Some(true),
        grace_period: Some(Period::from_str("20ms").unwrap()),
        on_change_strategy: Some(OnChangeStrategy::KillThenRestart),
        on_success: Some(Action::from_str("refresh").unwrap()),
        on_failure: Some(Action::from_str("play-sound(name=car-horn)").unwrap()),
        watch: Some(vec!["src".to_string(), "tests".to_string()]),
        show_changes_count: Some(true),
        sound: SoundConfig {
            enabled: Some(true),
            base_volume: Some(Volume::from_str("50").unwrap()),
        },
        workdir: Some(PathBuf::from("/path/to/workdir")),
        skin: Default::default(),
    };
    base_job.apply(&job_to_apply);
    dbg!(&base_job);
    assert_eq!(&base_job, &job_to_apply);
}

#[test]
fn test_load_env_from_file() {
    use std::fs;

    // Create a temporary file for testing
    let temp_dir = std::env::temp_dir();
    let env_file_path = temp_dir.join("test_env_file");

    let env_content = r#"
# This is a comment
VARIABLE1=value1
VARIABLE2="quoted value"
VARIABLE3='single quoted'

# Another comment
VARIABLE4=unquoted value
EMPTY_VARIABLE=
"#;

    fs::write(&env_file_path, env_content).unwrap();

    let env_vars = Job::load_env_from_file(&env_file_path, None).unwrap();

    assert_eq!(env_vars.get("VARIABLE1"), Some(&"value1".to_string()));
    assert_eq!(env_vars.get("VARIABLE2"), Some(&"quoted value".to_string()));
    assert_eq!(env_vars.get("VARIABLE3"), Some(&"single quoted".to_string()));
    assert_eq!(env_vars.get("VARIABLE4"), Some(&"unquoted value".to_string()));
    assert_eq!(env_vars.get("EMPTY_VARIABLE"), Some(&"".to_string()));
    assert!(!env_vars.contains_key("NONEXISTENT"));

    // Clean up
    fs::remove_file(&env_file_path).unwrap();
}

#[test]
fn test_env_file_integration() {
    use std::fs;

    // Create a temporary directory for the test
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("bacon_env_test");
    fs::create_dir_all(&test_dir).unwrap();

    // Create test .env file
    let env_file_path = test_dir.join(".env");
    let env_content = r#"
# Test env file
FILE_VAR_1=from_file
FILE_VAR_2="quoted from file"
CARGO_TERM_COLOR=never
RUST_BACKTRACE=0
"#;
    fs::write(&env_file_path, env_content).unwrap();

    // Test loading environment variables from file
    let loaded_vars = Job::load_env_from_file(&env_file_path, None).unwrap();

    assert_eq!(loaded_vars.get("FILE_VAR_1"), Some(&"from_file".to_string()));
    assert_eq!(loaded_vars.get("FILE_VAR_2"), Some(&"quoted from file".to_string()));
    assert_eq!(loaded_vars.get("CARGO_TERM_COLOR"), Some(&"never".to_string()));
    assert_eq!(loaded_vars.get("RUST_BACKTRACE"), Some(&"0".to_string()));

    // Test with relative path
    let relative_env_file = PathBuf::from(".env");
    let loaded_vars_relative = Job::load_env_from_file(&relative_env_file, Some(&test_dir)).unwrap();
    assert_eq!(loaded_vars, loaded_vars_relative);

    // Test job with env_file
    let mut job = Job {
        env_file: Some(relative_env_file),
        env: vec![
            ("DIRECT_VAR".to_string(), "direct_value".to_string()),
            ("CARGO_TERM_COLOR".to_string(), "always".to_string()), // Should override file
        ].into_iter().collect(),
        ..Default::default()
    };

    // Apply the job to itself to trigger env_file processing
    let job_copy = job.clone();
    job.apply(&job_copy);

    // Verify that direct env vars override env_file vars
    assert_eq!(job.env.get("DIRECT_VAR"), Some(&"direct_value".to_string()));
    assert_eq!(job.env.get("CARGO_TERM_COLOR"), Some(&"always".to_string())); // Should be overridden

    // Clean up
    fs::remove_dir_all(&test_dir).unwrap();
}
