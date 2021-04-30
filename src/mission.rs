use {
    crate::*,
    anyhow::*,
    cargo_metadata::MetadataCommand,
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    std::{collections::HashSet, env, fs, iter, path::PathBuf, process::Command},
};

pub struct MissionLocation {
    pub intended_dir: PathBuf,
    pub package_directory: PathBuf,
    pub cargo_toml_file: PathBuf,
    pub intended_is_package: bool,
}

impl MissionLocation {
    pub fn new(args: &Args) -> Result<Self> {
        let intended_dir = args
            .path
            .as_ref()
            .map_or_else(|| env::current_dir().unwrap(), PathBuf::from);
        let intended_dir: PathBuf = fs::canonicalize(&intended_dir)?;
        let mut package_directory = intended_dir.clone();
        let mut intended_is_package = true;
        let cargo_toml_file = loop {
            let cargo_toml_file = package_directory.join("Cargo.toml");
            if cargo_toml_file.exists() {
                break cargo_toml_file;
            }
            intended_is_package = false;
            package_directory = match package_directory.parent() {
                Some(dir) => dir.to_path_buf(),
                None => {
                    return Err(anyhow!(
                        "Cargo.toml file not found.\n\
                        bacon must be launched \n\
                        * in a rust project directory\n\
                        * or with a rust project directory given in argument\n\
                        (a rust project directory contains a Cargo.toml file or has such parent)\n\
                        "
                    ));
                }
            };
        };
        Ok(Self {
            intended_dir,
            package_directory,
            cargo_toml_file,
            intended_is_package,
        })
    }
    pub fn package_name(&self) -> String {
        self.package_directory
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
    pub fn package_config_path(&self) -> PathBuf {
        let local_path = self.package_directory.join("bacon.toml");
        let remote_path = dirs::config_dir().unwrap().join("bacon.toml");

        match (local_path.exists(), remote_path.exists()) {
            (true, _) => local_path,
            (false, true) => remote_path,
            (_, _) => local_path,
        }
    }
}

/// the description of the mission of bacon
/// after analysis of the args, env, and surroundings
#[derive(Debug)]
pub struct Mission {
    pub package_name: String,
    pub job_name: String,
    cargo_execution_directory: PathBuf,
    job: Job,
    files_to_watch: Vec<PathBuf>,
    directories_to_watch: Vec<PathBuf>,
    pub settings: Settings,
}

impl Mission {
    pub fn new(
        location: MissionLocation,
        package_config: &PackageConfig,
        job_name: Option<&str>,
        settings: Settings,
    ) -> Result<Self> {
        let add_all_src = location.intended_is_package;
        let (job_name, job) = package_config
            .get_job(job_name)
            .map(|(n, j)| (n.to_string(), j.clone()))?;
        let metadata = MetadataCommand::new()
            .manifest_path(&location.cargo_toml_file)
            .exec()?;
        let mut files_to_watch = Vec::new();
        let mut directories_to_watch = Vec::new();
        if !location.intended_is_package {
            directories_to_watch.push(location.intended_dir.to_path_buf());
        }
        for item in metadata.packages {
            if item.source.is_none() {
                let item_path = item
                    .manifest_path
                    .parent()
                    .expect("parent of a target folder is a root folder");
                if add_all_src {
                    let src_watch_iter = iter::once("src");
                    let other_watch_iter = job.watch.iter().map(String::as_ref);
                    for dir in src_watch_iter.chain(other_watch_iter) {
                        let full_path = item_path.join(dir);
                        if full_path.exists() {
                            directories_to_watch.push(full_path);
                        } else {
                            warn!("missing {} dir: {:?}", dir, full_path);
                        }
                    }
                }
                if item.manifest_path.exists() {
                    files_to_watch.push(item.manifest_path);
                } else {
                    warn!("missing manifest file: {:?}", item.manifest_path);
                }
            }
        }

        let cargo_execution_directory = location.package_directory.to_path_buf();
        let package_name = location.package_name();
        Ok(Mission {
            package_name,
            cargo_execution_directory,
            job_name,
            job,
            files_to_watch,
            directories_to_watch,
            settings,
        })
    }

    /// configure the watcher with files and directories to watch
    pub fn add_watchs(&self, watcher: &mut RecommendedWatcher) -> Result<()> {
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
        let mut tokens = self.job.command.iter();
        let mut command = Command::new(
            tokens.next().unwrap(), // implies a check in the job
        );
        let mut no_default_features_done = false;
        let mut features_done = false;
        let mut last_is_features = false;
        for arg in tokens {
            let mut arg = arg.to_string();
            if last_is_features {
                // arg is expected there to be the list of features
                features_done = true;
                match (&self.settings.features, self.settings.no_default_features) {
                    (Some(features), false) => {
                        // we take the features of both the job and the args
                        arg = merge_features(&arg, &features);
                    }
                    (Some(features), true) => {
                        // arg add features and remove the job ones
                        arg = features.clone();
                    }
                    (None, true) => {
                        // arg just remove the job features
                        arg = "".to_string()
                    }
                    (None, false) => {
                        // nothing to change
                    }
                }
                last_is_features = false;
            } else if arg == "--no-default-features" {
                no_default_features_done = true;
                last_is_features = false;
            } else if arg == "--features" {
                last_is_features = true;
            }
            command.arg(arg);
        }
        if self.settings.no_default_features && !no_default_features_done {
            command.arg("--no-default-features");
        }
        if !features_done {
            if let Some(features) = &self.settings.features {
                command.arg("--features");
                command.arg(features);
            }
        }
        command.current_dir(&self.cargo_execution_directory);
        debug!("command: {:#?}", &command);
        command
    }

    /// whether we need stdout and not just stderr
    pub fn need_stdout(&self) -> bool {
        self.job.need_stdout
    }
}

fn merge_features(a: &str, b: &str) -> String {
    let mut features = HashSet::new();
    for feature in a.split(',') {
        features.insert(feature);
    }
    for feature in b.split(',') {
        features.insert(feature);
    }
    features.iter().map(|&s| s).collect::<Vec<&str>>().join(",")
}
