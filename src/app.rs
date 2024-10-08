use {
    crate::*,
    anyhow::Result,
    crokey::*,
    crossbeam::channel::{
        bounded,
        select,
    },
    notify::event::{
        AccessKind,
        AccessMode,
        DataChange,
        EventKind,
        ModifyKind,
    },
    termimad::{
        EventSource,
        crossterm::event::Event,
    },
};

#[cfg(windows)]
use {
    crokey::key,
    termimad::crossterm::event::{
        MouseEvent,
        MouseEventKind,
    },
};

/// Run the mission and return the reference to the next job to run, if any
pub fn run(
    w: &mut W,
    mission: Mission,
    event_source: &EventSource,
) -> Result<Option<JobRef>> {
    let keybindings = mission.settings.keybindings.clone();
    let mut ignorer = time!(Info, mission.ignorer());
    let (watch_sender, watch_receiver) = bounded(0);
    let on_change_strategy = mission
        .job
        .on_change_strategy
        .or(mission.settings.on_change_strategy)
        .unwrap_or(OnChangeStrategy::WaitThenRestart);
    let mut watcher =
        notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
            Ok(we) => {
                match we.kind {
                    EventKind::Modify(ModifyKind::Metadata(_)) => {
                        debug!("ignoring metadata change");
                        return; // useless event
                    }
                    EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                        debug!("ignoring 'any' data change");
                        return; // probably useless event with no real change
                    }
                    EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                        debug!("close write event: {we:?}");
                    }
                    EventKind::Access(_) => {
                        debug!("ignoring access event: {we:?}");
                        return; // probably useless event
                    }
                    _ => {
                        debug!("notify event: {we:?}");
                    }
                }
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

    let mut executor = MissionExecutor::new(&mission)?;

    let analyzer = mission.analyzer();
    let mut state = AppState::new(mission)?;
    state.computation_starts();
    state.draw(w)?;

    let mut task_executor = executor.start(state.new_task())?; // first computation

    let user_events = event_source.receiver();
    let mut next_job: Option<JobRef> = None;
    #[allow(unused_mut)]
    loop {
        let mut action: Option<&Action> = None;
        select! {
            recv(watch_receiver) -> _ => {
                debug!("watch event received");
                if task_executor.is_in_grace_period() {
                    debug!("ignoring notify event in grace period");
                    continue;
                }
                state.receive_watch_event();
                if state.auto_refresh.is_enabled() {
                    if !state.is_computing() || on_change_strategy == OnChangeStrategy::KillThenRestart {
                        action = Some(&Action::Internal(Internal::ReRun));
                    }
                }
            }
            recv(executor.line_receiver) -> info => {
                if let Ok(info) = info {
                    match info {
                        CommandExecInfo::Line(line) => {
                            state.add_line(line);
                        }
                        CommandExecInfo::End { status } => {
                            info!("execution finished with status: {:?}", status);
                            // computation finished
                            let output = state.take_output().unwrap_or_default();
                            let cmd_result = CommandResult::build(output, status, analyzer, &state.mission)?;
                            state.set_result(cmd_result);
                            action = state.action();
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
            recv(user_events) -> user_event => {
                match user_event?.event {
                    Event::Resize(mut width, mut height) => {
                        state.resize(width, height);
                    }
                    Event::Key(key_event) => {
                        let key_combination = KeyCombination::from(key_event);
                        debug!("key combination pressed: {}", key_combination);
                        action = keybindings.get(key_combination);
                    }
                    #[cfg(windows)]
                    Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollDown, .. }) => {
                        action = keybindings.get(key!(down));
                    }
                    #[cfg(windows)]
                    Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollUp, .. }) => {
                        action = keybindings.get(key!(up));
                    }
                    _ => {}
                }
                event_source.unblock(false);
            }
        }
        if let Some(action) = action.take() {
            debug!("requested action: {action:?}");
            match action {
                Action::Export(export_name) => {
                    let export_name = export_name.to_string();
                    state
                        .mission
                        .settings
                        .exports
                        .do_named_export(&export_name, &state);
                    state
                        .messages
                        .push(Message::short(format!("Export *{}* done", &export_name)));
                }
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
                    Internal::Refresh => {
                        state.clear();
                        task_executor.die();
                        task_executor = state.start_computation(&mut executor)?;
                    }
                    Internal::ReRun => {
                        task_executor.die();
                        task_executor = state.start_computation(&mut executor)?;
                    }
                    Internal::ScopeToFailures => {
                        if let Some(scope) = state.failures_scope() {
                            info!("scoping to failures: {scope:#?}");
                            next_job = Some(JobRef::Scope(scope));
                            break;
                        } else {
                            warn!("no available failures scope");
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
                    Internal::ToggleBacktrace(level) => {
                        state.toggle_backtrace(level);
                        task_executor.die();
                        task_executor = state.start_computation(&mut executor)?;
                    }
                    Internal::Scroll(scroll_command) => {
                        let scroll_command = *scroll_command;
                        state.apply_scroll_command(scroll_command);
                    }
                    Internal::Pause => {
                        state.auto_refresh = AutoRefresh::Paused;
                    }
                    Internal::Unpause => {
                        if state.changes_since_last_job_start > 0 {
                            state.clear();
                            task_executor.die();
                            task_executor = state.start_computation(&mut executor)?;
                        }
                        state.auto_refresh = AutoRefresh::Enabled;
                    }
                    Internal::TogglePause => match state.auto_refresh {
                        AutoRefresh::Enabled => {
                            state.auto_refresh = AutoRefresh::Paused;
                        }
                        AutoRefresh::Paused => {
                            if state.changes_since_last_job_start > 0 {
                                state.clear();
                                task_executor.die();
                                task_executor = state.start_computation(&mut executor)?;
                            }
                            state.auto_refresh = AutoRefresh::Enabled;
                        }
                    },
                },
                Action::Job(job_ref) => {
                    next_job = Some((*job_ref).clone());
                    break;
                }
            }
        }
        state.draw(w)?;
    }
    task_executor.die();
    Ok(next_job)
}
