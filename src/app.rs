use {
    crate::*,
    anyhow::Result,
    crokey::CroKey,
    crossbeam::channel::{bounded, select},
    crossterm::event::Event,
    termimad::EventSource,
};

/// Run the mission and return the reference to the next
/// job to run, if any
pub fn run(
    w: &mut W,
    mission: Mission,
) -> Result<Option<JobRef>> {

    let keybindings = mission.settings.keybindings.clone();
    let (watch_sender, watch_receiver) = bounded(0);
    let mut watcher = notify::recommended_watcher(move |res| match res {
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

    let mut state = AppState::new(mission)?;
    state.computation_starts();
    state.draw(w)?;

    executor.start(state.new_task())?; // first computation

    let event_source = EventSource::new()?;
    let user_events = event_source.receiver();
    let mut next_job: Option<JobRef> = None;
    #[allow(unused_mut)]
    loop {
        select! {
            recv(user_events) -> user_event => {
                match user_event?.event {
                    Event::Resize(mut width, mut height) => {
                        // I don't know why but Crossterm seems to always report an
                        // understimated size on Windows
                        #[cfg(windows)]
                        {
                            width += 1;
                            height += 1;
                        }
                        state.resize(width, height);
                        state.draw(w)?;
                    }
                    Event::Key(key_event) => {
                        if let Some(action) = keybindings.get(key_event) {
                            debug!("key pressed: {}", CroKey::from(key_event));
                            debug!("requested action: {action:?}");
                            match action {
                                Action::Internal(internal) => {
                                    match internal {
                                        Internal::Back => {
                                            if !state.close_help() {
                                                next_job = Some(JobRef::Previous);
                                                break;
                                            }
                                        }
                                        Internal::Help => {
                                            state.toggle_help();
                                        }
                                        Internal::Quit => {
                                            executor.die()?;
                                            break;
                                        }
                                        Internal::ToggleSummary => {
                                            state.toggle_summary_mode();
                                        }
                                        Internal::ToggleWrap => {
                                            state.toggle_wrap_mode();
                                        }
                                        Internal::ToggleBacktrace => {
                                            state.toggle_backtrace();
                                            if let Err(e) = executor.start(state.new_task()) {
                                                debug!("error sending task: {}", e);
                                            } else {
                                                state.computation_starts();
                                            }
                                        }
                                        Internal::Scroll(scroll_command) => {
                                            state.apply_scroll_command(*scroll_command);
                                        }
                                    }
                                    state.draw(w)?;
                                }
                                Action::Job(job_ref) => {
                                    next_job = Some((*job_ref).clone());
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                event_source.unblock(false);
            }
            recv(watch_receiver) -> _ => {
                debug!("got a watcher event");
                if let Err(e) = executor.start(state.new_task()) {
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
                            if state.should_end() {
                                info!("mission ends on success");
                                next_job = Some(JobRef::Previous);
                                break;
                            } else {
                                debug!("no end");
                            }
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
    Ok(next_job)
}
