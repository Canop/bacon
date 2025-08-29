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
        path::{
            Path,
            PathBuf,
        },
    },
};

static DEFAULT_WATCHES: &[&str] = &[
    "Cargo.toml",
    "src",
    "tests",
    "benches",
    "examples",
    "build.rs",
];

/// information on the paths which are relevant for a mission
#[derive(Debug)]
pub struct Context {
    pub name: String,
    pub nature: ContextNature,
    /// The current package/project
    pub package_directory: PathBuf,
    /// The root of the workspace, only defined when it makes sense
    /// and it's different from the package directory.
    ///
    /// Today it's only obtained from cargo metadata but in the future
    /// it could be obtained from other kind of sources.
    pub workspace_root: Option<PathBuf>,
    cargo_mission_location: Option<CargoContext>,
    /// An optional path to watch, given at launch and overriding the settings
    /// of the Cargo.toml file, bacon.toml file, etc.
    pub path_to_watch: Option<PathBuf>,
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
        let package_directory = find_package_directory(args)?;

        let name = package_directory
            .file_name()
            .unwrap_or(package_directory.as_os_str())
            .to_string_lossy()
            .to_string();

        let path_to_watch = args.watch.as_ref().map(PathBuf::from);

        // A cargo project is one directly containing a Cargo.toml file.
        // When the project is a Cargo project, some additional rules apply and
        // the Cargo.toml file(s) is/are used to determine the paths to watch.
        let mut cargo_toml_file = package_directory.join("Cargo.toml");
        let nature = if cargo_toml_file.exists() && cargo_toml_file.is_file() {
            ContextNature::Cargo
        } else {
            ContextNature::Other
        };

        let mut workspace_root = None;
        let mut cargo_mission_location = None;
        if nature == ContextNature::Cargo {
            let metadata = if args.offline {
                MetadataCommand::new()
                    .current_dir(&package_directory)
                    .no_deps()
                    .other_options(["--frozen".to_string(), "--offline".to_string()])
                    .exec()
            } else {
                MetadataCommand::new()
                    .current_dir(&package_directory)
                    .exec()
            };
            let metadata = metadata?;
            if let Some(resolved_root) = metadata.resolve.and_then(|resolve| resolve.root) {
                cargo_toml_file = metadata
                    .packages
                    .iter()
                    .find(|p| p.id == resolved_root)
                    .map(|p| p.manifest_path.as_std_path().to_path_buf())
                    .expect("resolved manifest was not in package list");
                if metadata.workspace_root.as_std_path() != package_directory {
                    workspace_root = Some(metadata.workspace_root.as_std_path().to_path_buf());
                }
            }
            cargo_mission_location = Some(CargoContext {
                cargo_toml_file,
                packages: metadata.packages,
            });
        }

        Ok(Self {
            name,
            nature,
            package_directory,
            workspace_root,
            cargo_mission_location,
            path_to_watch,
        })
    }
    pub fn mission<'s>(
        &self,
        concrete_job_ref: ConcreteJobRef,
        leaf_job: &Job, // the raw job as defined, without using root settings
        settings: &'s Settings,
    ) -> Result<Mission<'s>> {
        // the real job used in the mission is built from settings.all_jobs
        // on which the provided leaf job is applied
        let mut job = settings.all_jobs.clone();
        job.apply(leaf_job);

        let location_name = self.name.clone();
        let mut paths_to_watch: Vec<PathBuf> = Vec::new();
        if let Some(path_to_watch) = &self.path_to_watch {
            paths_to_watch.push(path_to_watch.clone());
        } else {
            // Automatically watch all kinds of source files.
            // "watches", at this point, aren't full path, they still must be joined
            // with the right path which may depend on the
            let mut watches = Vec::new();
            if let Some(v) = &job.watch {
                for watch in v.iter() {
                    watches.push(watch.as_str());
                }
            }
            let add_default = job.default_watch.unwrap_or(true);
            if add_default {
                for watch in DEFAULT_WATCHES {
                    if !watches.contains(watch) {
                        watches.push(watch);
                    }
                }
            }
            add_to_paths_to_watch(&watches, &self.package_directory, &mut paths_to_watch);
            if let Some(workspace_root) = &self.workspace_root {
                // there's usually not much src at the workspace level but we must
                // at least watch the Cargo.toml file
                add_to_paths_to_watch(&watches, workspace_root, &mut paths_to_watch);
            }
            if let Some(location) = &self.cargo_mission_location {
                for item in &location.packages {
                    // if it's a local package
                    if item.source.is_none() {
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
        }

        let mut conf_execution_directory = job.workdir.as_ref();
        if let Some(path) = conf_execution_directory {
            if !path.exists() {
                error!("Ignoring configured non existing workdir: {path:?}");
                conf_execution_directory = None;
            }
        }
        let execution_directory = conf_execution_directory
            .unwrap_or(&self.package_directory)
            .to_path_buf();

        Ok(Mission {
            location_name,
            concrete_job_ref,
            execution_directory,
            package_directory: self.package_directory.clone(),
            workspace_directory: self.workspace_root.clone(),
            job,
            paths_to_watch,
            settings,
        })
    }
    pub fn workspace_cargo_path(&self) -> Option<PathBuf> {
        self.workspace_root.as_ref().map(|p| p.join("Cargo.toml"))
    }
    /// return the location of the workspace level bacon.toml file
    /// (if it's different from the package level bacon.toml file)
    pub fn workspace_config_path(&self) -> Option<PathBuf> {
        self.workspace_root.as_ref().map(|p| p.join("bacon.toml"))
    }
    pub fn workspace_dot_config_path(&self) -> Option<PathBuf> {
        self.workspace_root
            .as_ref()
            .map(|p| p.join(".config/bacon.toml"))
    }
    pub fn package_cargo_path(&self) -> PathBuf {
        self.package_directory.join("Cargo.toml")
    }
    pub fn package_config_path(&self) -> PathBuf {
        self.package_directory.join("bacon.toml")
    }
    pub fn package_dot_config_path(&self) -> PathBuf {
        self.package_directory.join(".config/bacon.toml")
    }
    pub fn unix_socket_path(&self) -> PathBuf {
        self.package_directory.join(".bacon.socket")
    }
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

/// The "package directory", unless specified with --project, is the closest
/// bacon.toml or Cargo.toml directory, or the current directory if none is found.
fn find_package_directory(args: &Args) -> Result<PathBuf> {
    if let Some(dir) = args.project.as_ref() {
        let path = PathBuf::from(dir);
        if !path.exists() || !path.is_dir() {
            bail!("The project path must be a directory");
        }
        return Ok(path);
    }
    let base_dir = env::current_dir().unwrap();
    let package_directory = closest_bacon_or_cargo_dir(&base_dir).unwrap_or(base_dir);
    Ok(package_directory)
}

fn closest_bacon_or_cargo_dir(start_path: &Path) -> Option<PathBuf> {
    let mut current_path = start_path;
    loop {
        let bacon_toml = current_path.join("bacon.toml");
        if bacon_toml.exists() && bacon_toml.is_file() {
            return Some(current_path.to_path_buf());
        }
        let cargo_toml = current_path.join("Cargo.toml");
        if cargo_toml.exists() && cargo_toml.is_file() {
            return Some(current_path.to_path_buf());
        }
        if let Some(parent) = current_path.parent() {
            current_path = parent;
        } else {
            break;
        }
    }
    None
}
