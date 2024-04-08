use {
    crate::*,
    anyhow::Result,
    lazy_regex::regex_replace_all,
    notify::{
        RecommendedWatcher,
        RecursiveMode,
        Watcher,
    },
    std::{
        collections::HashSet,
        path::PathBuf,
        process::Command,
    },
};

static DEFAULT_WATCHES: &[&str] = &["src", "tests", "benches", "examples"];

/// the description of the mission of bacon
/// after analysis of the args, env, and surroundings
#[derive(Debug)]
pub struct Mission<'s> {
    pub location_name: String,
    pub job_name: String,
    pub cargo_execution_directory: PathBuf,
    pub workspace_root: PathBuf,
    pub job: Job,
    files_to_watch: Vec<PathBuf>,
    directories_to_watch: Vec<PathBuf>,
    pub settings: &'s Settings,
}

impl<'s> Mission<'s> {
    pub fn new(
        location: &MissionLocation,
        job_name: String,
        job: Job,
        settings: &'s Settings,
    ) -> Result<Self> {
        let location_name = location.name();
        let add_all_src = location.intended_is_package;
        let mut files_to_watch: Vec<PathBuf> = Vec::new();
        let mut directories_to_watch = Vec::new();
        if !location.intended_is_package {
            directories_to_watch.push(location.intended_dir.clone());
        }
        for item in &location.packages {
            if item.source.is_none() {
                let item_path = item
                    .manifest_path
                    .parent()
                    .expect("parent of a target folder is a root folder");
                if add_all_src {
                    let mut watches: Vec<&str> = job.watch.iter().map(|s| s.as_str()).collect();
                    if job.default_watch {
                        for watch in DEFAULT_WATCHES {
                            if !watches.contains(watch) {
                                watches.push(watch);
                            }
                        }
                    }
                    debug!("watches: {watches:?}");
                    for dir in &watches {
                        let full_path = item_path.join(dir);
                        if full_path.exists() {
                            directories_to_watch.push(full_path.into());
                        } else {
                            info!("missing {} dir: {:?}", dir, full_path);
                        }
                    }
                }
                if item.manifest_path.exists() {
                    files_to_watch.push(item.manifest_path.clone().into());
                } else {
                    warn!("missing manifest file: {:?}", item.manifest_path);
                }
            }
        }

        let cargo_execution_directory = location.package_directory.clone();
        Ok(Mission {
            location_name,
            job_name,
            cargo_execution_directory,
            workspace_root: location.workspace_root.clone(),
            job,
            files_to_watch,
            directories_to_watch,
            settings,
        })
    }

    /// Return an Ignorer if required by the job's settings
    /// and if the mission takes place in a git repository
    pub fn ignorer(&self) -> Option<Ignorer> {
        match self.job.apply_gitignore {
            Some(false) => {
                debug!("No gitignorer because of settings");
                None
            }
            _ => {
                // by default we apply gitignore rules
                match Ignorer::new(&self.workspace_root) {
                    Ok(ignorer) => Some(ignorer),
                    Err(e) => {
                        // might be normal, eg not in a git repo
                        debug!("Failed to initialise git ignorer: {e}");
                        None
                    }
                }
            }
        }
    }

    /// Return the path to the bacon-locations file
    pub fn bacon_locations_path(&self) -> PathBuf {
        let path = &self.settings.export.path;
        if path.is_relative() {
            self.workspace_root.join(path)
        } else {
            path.to_path_buf()
        }
    }

    /// the action bound to success on this job
    pub fn on_success(&self) -> &Option<Action> {
        &self.job.on_success
    }

    pub fn is_success(
        &self,
        report: &Report,
    ) -> bool {
        report.is_success(self.job.allow_warnings, self.job.allow_failures)
    }

    /// configure the watcher with files and directories to watch
    pub fn add_watchs(
        &self,
        watcher: &mut RecommendedWatcher,
    ) -> Result<()> {
        for file in &self.files_to_watch {
            debug!("add watch file {:?}", file);
            watcher.watch(file, RecursiveMode::NonRecursive)?;
        }
        for dir in &self.directories_to_watch {
            debug!("add watch dir {:?}", dir);
            watcher.watch(dir, RecursiveMode::Recursive)?;
        }
        Ok(())
    }

    /// build (and doesn't call) the external cargo command
    pub fn get_command(&self) -> Command {
        let expanded;
        let command = if self.job.expand_env_vars {
            expanded = self
                .job
                .command
                .iter()
                .map(|token| {
                    regex_replace_all!(r"\$([A-Z0-9a-z_]+)", token, |whole: &str, name| {
                        match std::env::var(name) {
                            Ok(value) => value,
                            Err(_) => {
                                warn!("variable {whole} not found in env");
                                whole.to_string()
                            }
                        }
                    })
                    .to_string()
                })
                .collect();
            &expanded
        } else {
            &self.job.command
        };
        let mut tokens = command.iter();
        let mut command = Command::new(
            tokens.next().unwrap(), // implies a check in the job
        );
        let mut no_default_features_done = false;
        let mut features_done = false;
        let mut last_is_features = false;
        let mut tokens = tokens.chain(&self.settings.additional_job_args);
        let mut has_double_dash = false;
        while let Some(arg) = tokens.next() {
            if arg == "--" {
                // we'll defer addition of the following arguments to after
                // the addition of the features stuff, so that the features
                // arguments are given to the cargo command.
                has_double_dash = true;
                break;
            }
            if last_is_features {
                if self.settings.all_features {
                    debug!("ignoring features given along --all-features");
                } else {
                    features_done = true;
                    // arg is expected there to be the list of features
                    match (&self.settings.features, self.settings.no_default_features) {
                        (Some(features), false) => {
                            // we take the features of both the job and the args
                            command.arg("--features");
                            command.arg(merge_features(arg, features));
                        }
                        (Some(features), true) => {
                            // arg add features and remove the job ones
                            command.arg("--features");
                            command.arg(features);
                        }
                        (None, true) => {
                            // we pass no feature
                        }
                        (None, false) => {
                            // nothing to change
                            command.arg("--features");
                            command.arg(arg);
                        }
                    }
                }
                last_is_features = false;
            } else if arg == "--no-default-features" {
                no_default_features_done = true;
                last_is_features = false;
                command.arg(arg);
            } else if arg == "--features" {
                last_is_features = true;
            } else {
                command.arg(arg);
            }
        }
        if self.settings.no_default_features && !no_default_features_done {
            command.arg("--no-default-features");
        }
        if self.settings.all_features {
            command.arg("--all-features");
        }
        if !features_done {
            if let Some(features) = &self.settings.features {
                if self.settings.all_features {
                    debug!("not using features because of --all-features");
                } else {
                    command.arg("--features");
                    command.arg(features);
                }
            }
        }
        if has_double_dash {
            command.arg("--");
            for arg in tokens {
                command.arg(arg);
            }
        }
        command.current_dir(&self.cargo_execution_directory);
        command.envs(&self.job.env);
        debug!("command: {:#?}", &command);
        command
    }

    /// whether we need stdout and not just stderr
    pub fn need_stdout(&self) -> bool {
        self.job.need_stdout
    }
}

fn merge_features(
    a: &str,
    b: &str,
) -> String {
    let mut features = HashSet::new();
    for feature in a.split(',') {
        features.insert(feature);
    }
    for feature in b.split(',') {
        features.insert(feature);
    }
    features.iter().copied().collect::<Vec<&str>>().join(",")
}
