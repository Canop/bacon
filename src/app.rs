use {
    crate::*,
    anyhow::*,
    crossbeam::channel::{bounded, select},
    crossterm::event::{KeyCode::*, KeyEvent, KeyModifiers},
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
    std::{env, fs, path::PathBuf},
    termimad::{Event, EventSource},
};

pub fn run(w: &mut W, args: Args) -> Result<()> {
    let root_dir = args.root.unwrap_or_else(|| env::current_dir().unwrap());
    let root_dir: PathBuf = fs::canonicalize(&root_dir)?;
    info!("root_dir: {:?}", &root_dir);
    let src_dir = root_dir.join("src");
    let cargo_toml_file = root_dir.join("Cargo.toml");
    if !src_dir.exists() || !cargo_toml_file.exists() {
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
    state.computing = true;
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
    watcher.watch(src_dir, RecursiveMode::Recursive)?;
    watcher.watch(cargo_toml_file, RecursiveMode::NonRecursive)?;

    let executor = Executor::new(root_dir, args.clippy)?;
    executor.task_sender.send(())?; // first computation

    let event_source = EventSource::new()?;
    let user_events = event_source.receiver();
    loop {
        select! {
            recv(user_events) -> user_event => {
                match user_event? {
                    Event::Resize(width, height) => {
                        state.resize(width, height);
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
                            (Home, _) => { state.scroll(w, ScrollCommand::Top)?; }
                            (End, _) => { state.scroll(w, ScrollCommand::Bottom)?; }
                            (Up, _) => { state.scroll(w, ScrollCommand::Lines(-1))?; }
                            (Down, _) => { state.scroll(w, ScrollCommand::Lines(1))?; }
                            (PageUp, _) => { state.scroll(w, ScrollCommand::Pages(-1))?; }
                            (PageDown, _) => { state.scroll(w, ScrollCommand::Pages(1))?; }
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
                if let Err(e) = executor.task_sender.try_send(()) {
                    debug!("error sending task: {}", e);
                } else {
                    state.computing = true;
                    state.draw(w)?;
                }
            }
            recv(executor.line_receiver) -> line => {
                match line? {
                    Ok(Some(line)) => {
                        state.add_line(line);
                    }
                    Ok(None) => {
                        // computation finished
                        if let Some(lines) = state.take_lines() {
                            state.set_report(Report::from_err_lines(lines)?);
                        } else {
                            warn!("a computation finished but didn't start?");
                        }
                        state.computing = false;
                    }
                    Err(e) => {
                        warn!("error in computation: {}", e);
                        state.computing = false;
                        break;
                    }
                }
                state.draw(w)?;
            }
        }
    }
    Ok(())
}
