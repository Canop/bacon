use {
    crate::*,
    anyhow::*,
    crossbeam::channel::{bounded, select, unbounded, Receiver, Sender},
    crossterm::{
        cursor,
        event::{KeyCode::*, KeyEvent, KeyModifiers},
        execute,
        style::{Colorize, Styler},
        terminal, ExecutableCommand, QueueableCommand,
    },
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    serde::{Deserialize, Serialize},
    std::{env, fs, io::Write, path::PathBuf, process::Command},
    termimad::{Event, EventSource},
};

// Represents a subset of JSON returned by `cargo metadata`

#[derive(Serialize, Deserialize, Debug)]
pub struct CargoMetadata {
    #[serde(rename = "packages")]
    packages: Vec<Package>,

    #[serde(rename = "workspace_members")]
    workspace_members: Vec<String>,

    #[serde(rename = "target_directory")]
    target_directory: String,

    #[serde(rename = "version")]
    version: i64,

    #[serde(rename = "workspace_root")]
    workspace_root: String,

    #[serde(rename = "metadata")]
    metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "version")]
    version: String,

    #[serde(rename = "id")]
    id: String,

    #[serde(rename = "license")]
    license: Option<String>,

    #[serde(rename = "license_file")]
    license_file: Option<String>,

    #[serde(rename = "description")]
    description: Option<String>,

    #[serde(rename = "targets")]
    targets: Vec<Target>,

    #[serde(rename = "source")]
    source: Option<String>,

    #[serde(rename = "manifest_path")]
    manifest_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Target {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "src_path")]
    src_path: String,

    #[serde(rename = "edition")]
    edition: String,

    #[serde(rename = "doctest")]
    doctest: bool,

    #[serde(rename = "required-features")]
    required_features: Option<Vec<String>>,
}

fn find_folders_to_watch(root_dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let output = Command::new("cargo")
        .current_dir(root_dir)
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .output()?;

    let output_string = String::from_utf8(output.stdout)?;
    let metadata: CargoMetadata = serde_json::from_str(&output_string)?;

    let mut folders_to_watch = vec![];
    for item in metadata.packages {
        if item.source.is_none() {
            // We're only concerned with items where the source is None
            for target in item.targets {
                if target.src_path.ends_with("lib.rs") || target.src_path.ends_with("main.rs") {
                    // targets can contain the build script as well, so we need to eliminate them.

                    let target_folder = PathBuf::from_str(&target.src_path)?;
                    let target_folder_parent = PathBuf::from(
                        target_folder
                            .parent()
                            .expect("parent of target is a root folder."),
                    );
                    if target_folder_parent.ends_with("src") {
                        // to ensure misc binaries are not counted, such as
                        // src/binaries/foo_main.rs
                        folders_to_watch.push(target_folder_parent)
                    }
                }
            }
        }
    }
    Ok(folders_to_watch)
}

pub fn run(w: &mut W, args: Args) -> Result<()> {
    let root_dir = args.root.unwrap_or_else(|| env::current_dir().unwrap());
    let root_dir: PathBuf = fs::canonicalize(&root_dir)?;
    let src_dirs = find_folders_to_watch(&root_dir).unwrap();
    debug!("root_dir: {:?}", &root_dir);
    let cargo_toml_file = root_dir.join("Cargo.toml");
    if !cargo_toml_file.exists() {
        return Err(anyhow!(
            "bacon must be launched either\n\
            * in a rust project directory\n\
            * or with a rust project directory given in argument\n\
            (the rust project directory is the one with the Cargo.toml file and the src directory)\n\
            "
        ));
    }

    let mut state = AppState::new(&root_dir)?;
    if args.summary {
        state.summary = true;
    }
    let event_source = EventSource::new()?;
    let user_events = event_source.receiver();
    state.draw(w)?;
    state.report = Some(Report::compute(&root_dir, args.clippy)?);
    state.computing = false;
    state.draw(w)?;

    let (watch_sender, watch_receiver) = bounded(0);
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| match res {
        Ok(_) => {
            debug!("notify event received");
            if let Err(e) = watch_sender.send(()) {
                debug!("error when notifying on inotify event: {}", e);
            }
        }
        Err(e) => warn!("watch error: {:?}", e),
    })?;

    for src_dir in src_dirs.iter() {
        watcher.watch(src_dir, RecursiveMode::Recursive)?;
    }
    watcher.watch(cargo_toml_file, RecursiveMode::NonRecursive)?;

    let computer = Computer::new(root_dir, args.clippy)?;

    loop {
        select! {
            recv(user_events) -> user_event => {
                match user_event? {
                    Event::Resize(width, height) => {
                        state.screen = (width, height);
                        state.draw(w)?;
                    }
                    Event::Key(KeyEvent{ code, modifiers }) => {
                        match (code, modifiers) {
                            (Char('q'), KeyModifiers::NONE)
                                | (Char('c'), KeyModifiers::CONTROL)
                                | (Char('q'), KeyModifiers::CONTROL)
                            => {
                                debug!("user requests quit");
                                break;
                            }
                            (Char('s'), KeyModifiers::NONE) => {
                                debug!("user toggles summary mode");
                                state.summary ^= true;
                                state.draw(w)?;
                            }
                            _ => {
                                debug!("ignored key event: {:?}", user_event);
                            }
                        }
                    }
                    _ => {}
                }
                event_source.unblock(false);
            }
            recv(watch_receiver) -> _ => {
                debug!("got a watcher event");
                if let Err(e) = computer.task_sender.try_send(()) {
                    debug!("error sending task: {}", e);
                } else {
                    state.computing = true;
                    state.draw(w)?;
                }
            }
            recv(computer.report_receiver) -> report => {
                state.report = Some(report?);
                state.computing = false;
                state.draw(w)?;
            }
        }
    }
    Ok(())
}
