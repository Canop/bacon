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
    std::{env, io::Write, path::PathBuf},
    termimad::{Event, EventSource},
};

pub fn run(w: &mut W) -> Result<()> {
    let mut state = AppState::new()?;
    let event_source = EventSource::new()?;
    let user_events = event_source.receiver();
    state.draw(w)?;
    state.report = Some(Report::compute()?);
    state.computing = false;
    state.draw(w)?;

    let src_dir = env::current_dir()?.join("src");
    if !src_dir.exists() {
        return Err(anyhow!("src directory not found"));
    }
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

    let computer = Computer::new()?;

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
                            (Char('q'), KeyModifiers::NONE) | (Char('c'), KeyModifiers::CONTROL) => {
                                debug!("user requests quit");
                                break;
                            }
                            _ => {
                                debug!("ignored key event: {:?}", user_event);
                            }
                        }
                    }
                    _ => {
                    }
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
