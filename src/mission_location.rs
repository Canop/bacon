use {
    crate::*,
    anyhow::{
        bail,
        Result,
    },
    cargo_metadata::MetadataCommand,
    std::{
        env,
        fmt,
        fs,
        path::PathBuf,
    },
};

pub struct MissionLocation {
    pub intended_dir: PathBuf,
    pub workspace_root: PathBuf,
    pub package_directory: PathBuf,
    pub cargo_toml_file: PathBuf,
    pub intended_is_package: bool,
    pub packages: Vec<cargo_metadata::Package>,
}

impl fmt::Debug for MissionLocation {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("MissionLocation")
            .field("intended_dir", &self.intended_dir)
            .field("package_directory", &self.package_directory)
            .field("cargo_toml_file", &self.cargo_toml_file)
            .field("intended_is_package", &self.intended_is_package)
            .finish()
    }
}

impl MissionLocation {
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
        let metadata = match metadata {
            Ok(m) => m,
            Err(cargo_metadata::Error::CargoMetadata { stderr })
                if cargo_manifest_not_found(&stderr) =>
            {
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
        let workspace_root = metadata.workspace_root.clone().into();
        let cargo_toml_file;
        let package_directory;
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
            workspace_root,
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
