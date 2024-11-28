use {
    crate::*,
    anyhow::{
        Result,
        bail,
    },
    cargo_metadata::MetadataCommand,
    std::{
        env,
        fmt,
        fs,
        path::{
            Path,
            PathBuf,
        },
    },
};

static DEFAULT_WATCHES: &[&str] = &["src", "tests", "benches", "examples", "build.rs"];

/// information on the paths which are relevant for a mission
#[derive(Debug)]
pub struct Context {
    pub name: String,
    pub intended_dir: PathBuf,
    /// The current package
    pub package_directory: PathBuf,
    /// The root of the workspace, only defined when it makes sense
    /// and it's different from the package directory.
    ///
    /// Today it's only obtained from cargo metadata but in the future
    /// it could be obtained from other kind of sources.
    pub workspace_root: Option<PathBuf>,
    cargo_mission_location: Option<CargoContext>,
    /// When intended, the path given at launch, isn't a package, it means
    /// we don't want to watch the whole workspace but only the given path
    pub intended_is_package: bool,
}

/// Specific data for a cargo related mission
struct CargoContext {
    pub cargo_toml_file: PathBuf,
    pub packages: Vec<cargo_metadata::Package>,
}

impl fmt::Debug for CargoContext {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("CargoContext")
            .field("cargo_toml_file", &self.cargo_toml_file)
            .finish_non_exhaustive()
    }
}

impl Context {
    pub fn new(args: &Args) -> Result<Self> {
        let intended_dir = args
            .path
            .as_ref()
            .map_or_else(|| env::current_dir().unwrap(), PathBuf::from);
        let metadata = if args.offline {
            MetadataCommand::new()
                .current_dir(&intended_dir)
                .no_deps()
                .other_options(["--frozen".to_string(), "--offline".to_string()])
                .exec()
        } else {
            MetadataCommand::new().current_dir(&intended_dir).exec()
        };
        let need_cargo = false;
        let metadata = match metadata {
            Ok(m) => Some(m),
            Err(cargo_metadata::Error::CargoMetadata { stderr })
                if cargo_manifest_not_found(&stderr) =>
            {
                if need_cargo {
                    bail!(
                        "Cargo.toml file not found.\n\
                        bacon must be launched \n\
                        * in a rust project directory\n\
                        * or with a rust project directory given in argument\n\
                        (a rust project directory contains a Cargo.toml file or has such parent)\n\
                        "
                    );
                } else {
                    None
                }
            }
            Err(other) => bail!(other),
        };
        let package_directory;
        let workspace_root;
        let cargo_mission_location;
        let intended_is_package;
        if let Some(metadata) = metadata {
            // Cargo/Rust project
            let cargo_toml_file;
            if let Some(resolved_root) = metadata.resolve.and_then(|resolve| resolve.root) {
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
                workspace_root = if metadata.workspace_root.as_std_path() == package_directory {
                    None
                } else {
                    Some(metadata.workspace_root.as_std_path().to_path_buf())
                };
            } else {
                // resolved to a virtual manifest (of a workspace)
                package_directory = metadata.workspace_root.as_std_path().to_path_buf();
                cargo_toml_file = package_directory.join("Cargo.toml");
                workspace_root = None;
            }
            intended_is_package =
                fs::canonicalize(&intended_dir)? == fs::canonicalize(&package_directory)?;
            cargo_mission_location = Some(CargoContext {
                cargo_toml_file,
                packages: metadata.packages,
            });
        } else {
            // Non cargo project
            workspace_root = None;
            cargo_mission_location = None;
            package_directory = intended_dir.clone();
            intended_is_package = true;
        };
        let name = package_directory
            .file_name()
            .unwrap_or(package_directory.as_os_str())
            .to_string_lossy()
            .to_string();
        Ok(Self {
            name,
            intended_dir,
            package_directory,
            workspace_root,
            intended_is_package,
            cargo_mission_location,
        })
    }
    pub fn mission<'s>(
        &self,
        concrete_job_ref: ConcreteJobRef,
        job: Job,
        settings: &'s Settings,
    ) -> Result<Mission<'s>> {
        let location_name = self.name.clone();
        // We don't need to make the difference between a file and a dir, this can
        // be determined by the watcher

        // "watches", at this point, aren't full path, they still must be joined
        // with the right path which may depend on the
        let mut watches: Vec<&str> = job
            .watch
            .as_ref()
            .unwrap_or(&settings.watch)
            .iter()
            .map(|s| s.as_str())
            .collect();
        let add_default = job.default_watch.unwrap_or(settings.default_watch);
        if add_default {
            for watch in DEFAULT_WATCHES {
                if !watches.contains(watch) {
                    watches.push(watch);
                }
            }
        }
        debug!("watches: {watches:?}");

        let mut paths_to_watch: Vec<PathBuf> = Vec::new();

        // when bacon is given a path at launch, and when this path isn't a
        // path to a package, the goal is to specify that this path should
        // be watched, wich allows not watching everything in the workspace
        if self.intended_is_package {
            // automatically watch all kinds of source files
            add_to_paths_to_watch(&watches, &self.intended_dir, &mut paths_to_watch);
            if let Some(workspace_root) = &self.workspace_root {
                add_to_paths_to_watch(&watches, workspace_root, &mut paths_to_watch);
            }
            if let Some(location) = &self.cargo_mission_location {
                for item in &location.packages {
                    if item.source.is_none() {
                        // FIXME why this check
                        let item_path = item
                            .manifest_path
                            .parent()
                            .expect("parent of a target folder is a root folder");
                        add_to_paths_to_watch(
                            &watches,
                            item_path.as_std_path(),
                            &mut paths_to_watch,
                        );
                        if item.manifest_path.exists() {
                            paths_to_watch.push(item.manifest_path.clone().into());
                        } else {
                            warn!("missing manifest file: {:?}", item.manifest_path);
                        }
                    }
                }
            }
        } else {
            // we only watch the given "intended" path
            paths_to_watch.push(self.intended_dir.clone());
        }
        let root_directory = self
            .workspace_root
            .as_ref()
            .unwrap_or(&self.package_directory)
            .clone();

        let execution_directory = self.package_directory.clone();
        Ok(Mission {
            location_name,
            concrete_job_ref,
            execution_directory,
            package_directory: self.package_directory.clone(),
            root_directory,
            job,
            paths_to_watch,
            settings,
        })
    }
    pub fn package_config_path(&self) -> PathBuf {
        self.package_directory.join("bacon.toml")
    }
    /// return the location of the workspace level bacon.toml file
    /// (it may be the same path than the package config)
    pub fn workspace_config_path(&self) -> Option<PathBuf> {
        self.workspace_root.as_ref().map(|p| p.join("bacon.toml"))
    }
}

fn cargo_manifest_not_found(err: &str) -> bool {
    err.starts_with("error: could not find `Cargo.toml`")
}

fn add_to_paths_to_watch(
    watches: &[&str],
    base_path: &Path,
    paths_to_watch: &mut Vec<PathBuf>,
) {
    for watch in watches {
        let full_path = base_path.join(watch);
        if !paths_to_watch.contains(&full_path) && full_path.exists() {
            paths_to_watch.push(full_path);
        }
    }
}
