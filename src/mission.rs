use {
    crate::*,
    anyhow::*,
    cargo_metadata::MetadataCommand,
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    std::{
        env,
        fs,
        path::PathBuf,
    },
};

/// the description of the mission of bacon
/// after analysis of the args, env, and surroundings
#[derive(Debug)]
pub struct Mission {
    pub name: String,
    cargo_execution_directory: PathBuf,
    cargo_command_exe: String,
    cargo_command_args: Vec<String>,
    files_to_watch: Vec<PathBuf>,
    directories_to_watch: Vec<PathBuf>,
    pub display_settings: DisplaySettings,
}

impl Mission {
    pub fn from(args: Args) -> Result<Self> {
        let display_settings = DisplaySettings::from(&args);
        let intended_dir = args.root.unwrap_or_else(|| env::current_dir().unwrap());
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

        let add_all_src = intended_is_package;
        let metadata = MetadataCommand::new()
            .manifest_path(&cargo_toml_file)
            .exec()?;
        let mut files_to_watch = Vec::new();
        let mut directories_to_watch = Vec::new();
        if !intended_is_package {
            directories_to_watch.push(intended_dir);
        }
        for item in metadata.packages {
            if item.source.is_none() {
                let item_path = item
                    .manifest_path
                    .parent()
                    .expect("parent of a target folder is a root folder");
                if add_all_src {
                    directories_to_watch.push(item_path.join("src"));
                }
                files_to_watch.push(item.manifest_path);
            }
        }

        let cargo_execution_directory = package_directory.to_path_buf();
        let name = package_directory.file_name().unwrap().to_string_lossy().to_string();
        let cargo_command_exe = "cargo".to_string();
        let sub_command = if args.clippy { "clippy" } else { "check" };
        let cargo_command_args = vec![
            sub_command.to_string(),
            "--color".to_string(),
            "always".to_string(),
        ];
        Ok(Mission {
            name,
            cargo_execution_directory,
            cargo_command_exe,
            cargo_command_args,
            files_to_watch,
            directories_to_watch,
            display_settings,
        })
    }

    /// configure the watcher with files and directories to watch
    pub fn add_watchs(&self, watcher: &mut RecommendedWatcher) -> Result<()> {
        for file in &self.files_to_watch {
            watcher.watch(file, RecursiveMode::NonRecursive)?;
        }
        for dir in &self.directories_to_watch {
            watcher.watch(dir, RecursiveMode::Recursive)?;
        }
        Ok(())
    }

    /// build (and doesn't call) the external cargo command
    pub fn get_command(&self) -> Command {
        let mut command = Command::new("cargo");
        for arg in &self.cargo_command_args {
            command.arg(arg);
        }
        command.current_dir(&self.cargo_execution_directory);
        command
    }
}

