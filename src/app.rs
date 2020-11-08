use {
    crate::*,
    anyhow::*,
    crossbeam::channel::{bounded, select},
    crossterm::event::{KeyCode::*, KeyEvent, KeyModifiers},
    notify::{RecommendedWatcher, Watcher},
    termimad::{Event, EventSource},
};

pub fn run(w: &mut W, mission: Mission) -> Result<()> {
    let mut state = AppState::new(&mission)?;
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
    mission.add_watchs(&mut watcher)?;

    let executor = Executor::new(mission.get_command())?;
    executor.start()?; // first computation

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
                                executor.die()?;
                                debug!("executor dead");
                                break;
                            }
                            (Char('s'), KeyModifiers::NONE) => {
                                debug!("user toggles summary mode");
                                state.toggle_summary_mode();
                                state.draw(w)?;
                            }
                            (Home, _) => { state.scroll(w, ScrollCommand::Top)?; }
                            (End, _) => { state.scroll(w, ScrollCommand::Bottom)?; }
                            (Up, _) => { state.scroll(w, ScrollCommand::Lines(-1))?; }
                            (Down, _) => { state.scroll(w, ScrollCommand::Lines(1))?; }
                            (PageUp, _) => { state.scroll(w, ScrollCommand::Pages(-1))?; }
                            (PageDown, _) => { state.scroll(w, ScrollCommand::Pages(1))?; }
                            (Char(' '), _) => { state.scroll(w, ScrollCommand::Pages(1))?; }
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
                if let Err(e) = executor.start() {
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
