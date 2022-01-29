use {
    crate::*,
    anyhow::{bail, Result},
    cargo_metadata::MetadataCommand,
    std::{
        env,
        fs,
        path::PathBuf,
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
            .unwrap_or_else(|| self.package_directory.as_os_str())
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

