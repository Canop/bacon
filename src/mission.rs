use {
    crate::*,
    anyhow::{bail, Result},
    cargo_metadata::MetadataCommand,
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    std::{
        collections::HashSet,
        env,
        fs,
        iter,
        path::PathBuf,
        process::Command,
    },
};

pub struct MissionLocation {
    pub intended_dir: PathBuf,
    pub package_directory: PathBuf,
    pub cargo_toml_file: PathBuf,
    pub intended_is_package: bool,
    pub packages: Vec<cargo_metadata::Package>,
}

impl MissionLocation {
    pub fn new(args: &Args) -> Result<Self> {
        let intended_dir = args
            .path
            .as_ref()
            .map_or_else(|| env::current_dir().unwrap(), PathBuf::from);
        let metadata = match MetadataCommand::new().current_dir(&intended_dir).exec() {
            Ok(m) => m,
            Err(cargo_metadata::Error::CargoMetadata { stderr }) if cargo_manifest_not_found(&stderr) => {
                bail!(
                    "Cargo.toml file not found.\n\
                    bacon must be launched \n\
                    * in a rust project directory\n\
                    * or with a rust project directory given in argument\n\
                    (a rust project directory contains a Cargo.toml file or has such parent)\n\
                    "
                );
            }
            Err(other) => bail!(other),
        };
        let resolve = metadata
            .resolve
            .expect("cargo metadata should resolve workspace without --no-deps");
        let cargo_toml_file;
        let package_directory;
        if let Some(resolved_root) = resolve.root {
            // resolved to a single package
            cargo_toml_file = metadata
                .packages
                .iter()
                .find(|p| p.id == resolved_root)
                .map(|p| p.manifest_path.as_std_path().to_path_buf())
                .expect("resolved manifest was not in package list");
            package_directory = cargo_toml_file
                .parent()
                .expect("file has no parent")
                .to_path_buf();
        } else {
            // resolved to a virtual manifest (of a workspace)
            package_directory = metadata.workspace_root.as_std_path().to_path_buf();
            cargo_toml_file = package_directory.join("Cargo.toml");
        }
        let intended_is_package =
            fs::canonicalize(&intended_dir)? == fs::canonicalize(&package_directory)?;
        Ok(Self {
            intended_dir,
            package_directory,
            cargo_toml_file,
            intended_is_package,
            packages: metadata.packages,
        })
    }
    pub fn name(&self) -> String {
        self.package_directory
            .file_name()
            .unwrap_or(self.package_directory.as_os_str())
            .to_string_lossy()
            .to_string()
    }
    pub fn package_config_path(&self) -> PathBuf {
        self.package_directory.join("bacon.toml")
    }
}

fn cargo_manifest_not_found(err: &str) -> bool {
    err.starts_with("error: could not find `Cargo.toml`")
}

/// the description of the mission of bacon
/// after analysis of the args, env, and surroundings
#[derive(Debug)]
pub struct Mission {
    pub location_name: String,
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
        let location_name = location.name();
        let add_all_src = location.intended_is_package;
        let (job_name, job) = package_config
            .get_job(job_name)
            .map(|(n, j)| (n.to_string(), j.clone()))?;
        let mut files_to_watch: Vec<PathBuf> = Vec::new();
        let mut directories_to_watch = Vec::new();
        if !location.intended_is_package {
            directories_to_watch.push(location.intended_dir);
        }
        for item in location.packages {
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
                            directories_to_watch.push(full_path.into());
                        } else {
                            warn!("missing {} dir: {:?}", dir, full_path);
                        }
                    }
                }
                if item.manifest_path.exists() {
                    files_to_watch.push(item.manifest_path.into());
                } else {
                    warn!("missing manifest file: {:?}", item.manifest_path);
                }
            }
        }

        let cargo_execution_directory = location.package_directory;
        Ok(Mission {
            location_name,
            job_name,
            cargo_execution_directory,
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
                            command.arg(&features);
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
    features.iter().copied().collect::<Vec<&str>>().join(",")
}
