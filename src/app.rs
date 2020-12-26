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
    state.computation_starts();
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

    let executor = Executor::new(&mission)?;
    executor.start()?; // first computation

    let event_source = EventSource::new()?;
    let user_events = event_source.receiver();
    let vim_keys = mission.settings.vim_keys;
    loop {
        select! {
            recv(user_events) -> user_event => {
                debug!("key event: {:?}", user_event);
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
                            (Char('w'), KeyModifiers::NONE) => {
                                debug!("user toggles wrapping");
                                state.toggle_wrap_mode();
                                state.draw(w)?;
                            }
                            (Home, _) => { state.scroll(w, ScrollCommand::Top)?; }
                            (End, _) => { state.scroll(w, ScrollCommand::Bottom)?; }
                            (Up, _) => { state.scroll(w, ScrollCommand::Lines(-1))?; }
                            (Down, _) => { state.scroll(w, ScrollCommand::Lines(1))?; }
                            (PageUp, _) => { state.scroll(w, ScrollCommand::Pages(-1))?; }
                            (PageDown, _) => { state.scroll(w, ScrollCommand::Pages(1))?; }
                            (Char(' '), _) => { state.scroll(w, ScrollCommand::Pages(1))?; }

                            (Char('g'), KeyModifiers::NONE) if vim_keys => {
                                state.scroll(w, ScrollCommand::Top)?;
                            }
                            (Char('G'), KeyModifiers::SHIFT) if vim_keys => {
                                state.scroll(w, ScrollCommand::Bottom)?;
                            }
                            (Char('k'), KeyModifiers::NONE) if vim_keys => {
                                state.scroll(w, ScrollCommand::Lines(-1))?;
                            }
                            (Char('j'), KeyModifiers::NONE) if vim_keys => {
                                state.scroll(w, ScrollCommand::Lines(1))?;
                            }
                            _ => {}
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
                    state.computation_starts();
                    state.draw(w)?;
                }
            }
            recv(executor.line_receiver) -> info => {
                match info? {
                    CommandExecInfo::Line(line) => {
                        state.add_line(line);
                        if !state.has_report() {
                            state.draw(w)?;
                        }
                    }
                    CommandExecInfo::End { status } => {
                        info!("execution finished with status: {:?}", status);
                        // computation finished
                        if let Some(lines) = state.take_lines() {
                            let cmd_result = CommandResult::new(lines, status)?;
                            state.set_result(cmd_result);
                        } else {
                            warn!("a computation finished but didn't start?");
                            state.computation_stops();
                        }
                        state.draw(w)?;
                    }
                    CommandExecInfo::Error(e) => {
                        warn!("error in computation: {}", e);
                        state.computation_stops();
                        state.draw(w)?;
                        break;
                    }
                    CommandExecInfo::Interruption => {
                        debug!("command was interrupted (by us)");
                    }
                }
            }
        }
    }
    Ok(())
}
