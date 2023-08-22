use {
    crate::*,
    anyhow::Result,
    crokey::CroKey,
    crossbeam::channel::{
        bounded,
        select,
    },
    termimad::crossterm::event::Event,
    termimad::EventSource,
};
#[cfg(windows)]
use termimad::crossterm::event::{
    MouseEventKind,
    KeyCode,
    KeyEvent,
    KeyModifiers,
};

/// Run the mission and return the reference to the next
/// job to run, if any
pub fn run(
    w: &mut W,
    mission: Mission,
    event_source: &EventSource,
) -> Result<Option<JobRef>> {
    let keybindings = mission.settings.keybindings.clone();
    let mut ignorer = time!(Info, mission.ignorer());
    let (watch_sender, watch_receiver) = bounded(0);
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(we) => {
                debug!("notify event: {we:?}");
                if let Some(ignorer) = ignorer.as_mut() {
                    match time!(Info, ignorer.excludes_all(&we.paths)) {
                        Ok(true) => {
                            debug!("all excluded");
                            return;
                        }
                        Ok(false) => {
                            debug!("at least one is included");
                        }
                        Err(e) => {
                            warn!("exclusion check failed: {e}");
                        }
                    }
                }
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

    let user_events = event_source.receiver();
    let mut next_job: Option<JobRef> = None;
    #[allow(unused_mut)]
    loop {
        let mut action: Option<&Action> = None;
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
                    }
                    Event::Key(key_event) => {
                        debug!("key pressed: {}", CroKey::from(key_event));
                        action = keybindings.get(key_event);
                    }
                    #[cfg(windows)]
                    Event::Mouse(mouse_event) => {
                        let key_code =
                        match mouse_event.kind {
                            MouseEventKind::ScrollDown => {KeyCode::Down}
                            MouseEventKind::ScrollUp => {KeyCode::Up}
                            _ => {KeyCode::Null}
                        };
                        action = keybindings.get(KeyEvent::new(key_code,KeyModifiers::NONE));
                    }
                    #[cfg(not(windows))]
                    _ => {}
                }
                event_source.unblock(false);
            }
            recv(watch_receiver) -> _ => {
                if let Err(e) = executor.start(state.new_task()) {
                    debug!("error sending task: {}", e);
                } else {
                    state.computation_starts();
                }
            }
            recv(executor.line_receiver) -> info => {
                match info? {
                    CommandExecInfo::Line(line) => {
                        state.add_line(line);
                    }
                    CommandExecInfo::End { status } => {
                        info!("execution finished with status: {:?}", status);
                        // computation finished
                        if let Some(output) = state.take_output() {
                            let cmd_result = CommandResult::new(output, status)?;
                            state.set_result(cmd_result);
                            action = state.action();
                        } else {
                            warn!("a computation finished but didn't start?");
                            state.computation_stops();
                        }
                    }
                    CommandExecInfo::Error(e) => {
                        warn!("error in computation: {}", e);
                        state.computation_stops();
                        break;
                    }
                    CommandExecInfo::Interruption => {
                        debug!("command was interrupted (by us)");
                    }
                }
            }
        }
        if let Some(action) = action.take() {
            debug!("requested action: {action:?}");
            match action {
                Action::Internal(internal) => match internal {
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
                        break;
                    }
                    Internal::ReRun => {
                        if let Err(e) = executor.start(state.new_task()) {
                            debug!("error sending task on re-rerun: {}", e);
                        } else {
                            state.computation_starts();
                        }
                    }
                    Internal::ToggleRawOutput => {
                        state.toggle_raw_output();
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
                        let scroll_command = *scroll_command;
                        state.apply_scroll_command(scroll_command);
                    }
                },
                Action::Job(job_ref) => {
                    next_job = Some((*job_ref).clone());
                    break;
                }
            }
        }
        state.draw(w)?;
    }
    executor.die()?;
    Ok(next_job)
}
