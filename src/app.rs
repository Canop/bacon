use {
    crate::*,
    anyhow::*,
    cargo_metadata::{Metadata, MetadataCommand},
    crossbeam::channel::{bounded, select, unbounded, Receiver, Sender},
    crossterm::{
        cursor,
        event::{KeyCode::*, KeyEvent, KeyModifiers},
        execute,
        style::{Colorize, Styler},
        terminal, ExecutableCommand, QueueableCommand,
    },
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    std::{env, fs, io::Write, path::PathBuf, process::Command},
    termimad::{Event, EventSource},
};

fn find_folders_to_watch(cargo_filepath: &PathBuf) -> Result<Vec<PathBuf>> {
    let metadata = MetadataCommand::new()
        .manifest_path(cargo_filepath)
        .exec()?;

    let mut paths_to_watch = vec![];

    for item in metadata.packages {
        if item.source.is_none() {
        let target_path = item
            .manifest_path
            .parent()
            .expect("parent of a target folder is a root folder");
        let mut target_pathbuf = PathBuf::from(target_path);
        target_pathbuf.push("src");
        paths_to_watch.push(target_pathbuf); // the src directory
        paths_to_watch.push(item.manifest_path); // the manifest_path
        }
    }
    Ok(paths_to_watch)
}

pub fn run(w: &mut W, args: Args) -> Result<()> {
    let root_dir = args.root.unwrap_or_else(|| env::current_dir().unwrap());
    let root_dir: PathBuf = fs::canonicalize(&root_dir)?;
    debug!("root_dir: {:?}", &root_dir);

    let cargo_toml_file = root_dir.join("Cargo.toml");
    let paths_to_watch = find_folders_to_watch(&cargo_toml_file).expect("Getting cargo metadata");

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

    for path_ in paths_to_watch.iter() {
        debug!("watching {:?}", path_);
        watcher.watch(path_, RecursiveMode::Recursive)?;
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
