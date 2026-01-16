use {
    crate::*,
    anyhow::Result,
    crokey::*,
    std::{
        io::Write,
        time::Duration,
    },
    termimad::{
        EventSource,
        EventSourceOptions,
        Ticker,
        crossbeam::channel::{
            Receiver,
            select,
        },
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

enum DoAfterMission {
    NextJob(JobRef),
    ReloadConfig,
    Quit,
}

impl From<JobRef> for DoAfterMission {
    fn from(job_ref: JobRef) -> Self {
        Self::NextJob(job_ref)
    }
}

/// Run the application until the user quits
pub fn run(
    w: &mut W,
    mut settings: Settings,
    args: &Args,
    context: &Context,
    headless: bool,
) -> Result<()> {
    let mut app_state = AppState {
        headless,
        ..Default::default()
    };
    let event_source = if headless {
        // in headless mode, in some contexts, ctrl-c might not be enough to kill
        // bacon so we add this handler
        ctrlc::set_handler(move || {
            eprintln!("bye");
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");
        None
    } else {
        Some(EventSource::with_options(EventSourceOptions {
            combine_keys: false,
            ..Default::default()
        })?)
    };
    #[allow(unused_variables)]
    let (action_tx, action_rx) = termimad::crossbeam::channel::unbounded();
    #[cfg(unix)]
    let _server = if settings.listen {
        Some(Server::new(context, action_tx.clone())?)
    } else {
        None
    };
    let mut job_stack = JobStack::default();
    let mut next_job = JobRef::Initial;
    let mut message = None;
    loop {
        let Some((concrete_job_ref, job)) = job_stack.pick_job(&next_job, &settings)? else {
            break;
        };
        let mission = context.mission(concrete_job_ref, &job, &settings)?;
        let do_after = app::run_mission(
            w,
            &mut app_state,
            mission,
            event_source.as_ref(),
            action_rx.clone(),
            message.take(),
        )?;
        match do_after {
            DoAfterMission::NextJob(job_ref) => {
                next_job = job_ref;
            }
            DoAfterMission::ReloadConfig => match Settings::read(args, context) {
                Ok(new_settings) => {
                    settings = new_settings;
                    message = Some(Message::short("Config reloaded"));
                }
                Err(e) => {
                    message = Some(Message::short(format!("Invalid config: {e}")));
                }
            },
            DoAfterMission::Quit => {
                break;
            }
        }
    }
    Ok(())
}

/// Run the mission and return what to do afterwards
fn run_mission(
    w: &mut W,
    app_state: &mut AppState,
    mission: Mission,
    event_source: Option<&EventSource>,
    action_rx: Receiver<Action>,
    message: Option<Message>,
) -> Result<DoAfterMission> {
    let headless = app_state.headless;
    let keybindings = mission.settings.keybindings.clone();
    let grace_period = mission.job.grace_period();

    let sound_player = mission.sound_player_if_needed();
    let mut sound_not_enabled_message_already_displayed = false;

    // build the watcher detecting and transmitting mission file changes
    let ignorer = time!(Info, mission.ignorer());
    let mission_watcher = Watcher::new(&mission.paths_to_watch, ignorer)?;

    // create the watcher for config file changes
    let config_watcher = Watcher::new(&mission.settings.config_files, IgnorerSet::default())?;

    // create the executor, mission, and state
    let mut executor = MissionExecutor::new(&mission)?;
    let on_change_strategy = mission.job.on_change_strategy();
    let mut mission_state = MissionState::new(app_state, mission)?;
    if let Some(message) = message {
        mission_state.messages.push(message);
    }
    mission_state.computation_starts();
    if !headless {
        mission_state.draw(w)?;
    }
    let mut task_executor = executor.start(mission_state.new_task())?; // first computation

    // A very low frequency tick generator, to ensure "config loaded" message doesn't stick
    // too long on the screen
    let mut ticker = Ticker::new();
    ticker.tick_infinitely((), Duration::from_secs(5));

    let _dummy_sender;
    let user_events = if let Some(event_source) = event_source {
        event_source.receiver()
    } else {
        let (sender, receiver) = termimad::crossbeam::channel::unbounded();
        _dummy_sender = sender;
        receiver
    };
    let mut mission_end = None;
    // loop on events
    #[allow(unused_mut)]
    loop {
        // The actions to execute in response to the event.
        // While it's a vec, action execution will stop at the first one quitting the
        // mission or requesting a task execution, and the rest of the vec will be dropped.
        let mut actions: Vec<Action> = Vec::new();
        select! {
            recv(ticker.tick_receiver) -> _ => {
                // just redraw
            }
            recv(mission_watcher.receiver) -> _ => {
                debug!("watch event received");
                if task_executor.is_in_grace_period() {
                    debug!("ignoring notify event in grace period");
                    continue;
                }
                mission_state.receive_watch_event();
                if mission_state.auto_refresh.is_enabled() {
                    if !mission_state.is_computing() || on_change_strategy == OnChangeStrategy::KillThenRestart {
                        actions.push(Action::ReRun);
                    }
                }
            }
            recv(config_watcher.receiver) -> _ => {
                info!("config watch event received");
                if mission_state.auto_refresh.is_enabled() {
                    grace_period.sleep(); // Fix #310
                    actions.push(Action::ReloadConfig);
                }
            }
            recv(executor.line_receiver) -> info => {
                if let Ok(info) = info {
                    match info {
                        CommandExecInfo::Line(line) => {
                            if headless {
                                match line.origin {
                                    CommandStream::StdOut => print!("{}", line.content),
                                    CommandStream::StdErr => eprint!("{}", line.content),
                                }
                            }
                            let line = line.into();
                            mission_state.add_line(line);
                        }
                        CommandExecInfo::End { status } => {
                            // computation finished
                            info!("execution finished with status: {status:?}");
                            mission_state.finish_task(status)?;
                            if headless {
                                for badge in mission_state.job_badges() {
                                    badge.draw(w)?;
                                }
                                writeln!(w)?;
                                w.flush()?;
                            }
                            if mission_state.is_success() {
                                if let Some(action) = &mission_state.mission.job.on_success {
                                    actions.push(action.clone());
                                }
                            }
                            if mission_state.is_failure() {
                                if let Some(action) = &mission_state.mission.job.on_failure {
                                    actions.push(action.clone());
                                }
                            }
                            if mission_state.changes_since_last_job_start > 0 && mission_state.auto_refresh.is_enabled() {
                                // will be ignored if a on_success or on_failures ends the mission
                                // or does a rerun already
                                actions.push(Action::ReRun);
                            }
                        }
                        CommandExecInfo::Error(e) => {
                            mission_state.computation_stops();
                            return Err(e.context(format!("error in computation for job '{}'", mission_state.mission.concrete_job_ref.badge_label())));
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
                        mission_state.resize(width, height);
                    }
                    Event::Key(key_event) => {
                        let key_combination = KeyCombination::from(key_event);
                        debug!("key combination pressed: {key_combination}");
                        if let Some(action) =  mission_state.on_key(key_combination) {
                            actions.push(action);
                        } else if let Some(action) = keybindings.get(key_combination) {
                            actions.push(action.clone());
                        }
                    }
                    #[cfg(windows)]
                    Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollDown, .. }) => {
                        let action = keybindings.get(key!(down));
                        if let Some(action) = action {
                            actions.push(action.clone());
                        }
                    }
                    #[cfg(windows)]
                    Event::Mouse(MouseEvent { kind: MouseEventKind::ScrollUp, .. }) => {
                        let action = keybindings.get(key!(up));
                        if let Some(action) = action {
                            actions.push(action.clone());
                        }
                    }
                    _ => {}
                }
                if let Some(event_source) = event_source {
                    event_source.unblock(false);
                }
            }
            recv(action_rx) -> action => {
                actions.push(action?);
            }
        }
        for action in actions.drain(..) {
            let mut rerun = false;
            debug!("requested action: {action:?}");
            match action {
                Action::Back => {
                    if !mission_state.back() {
                        mission_end = Some(DoAfterMission::NextJob(JobRef::Previous));
                    }
                }
                Action::BackOrQuit => {
                    if !mission_state.back() {
                        mission_end = Some(DoAfterMission::NextJob(JobRef::PreviousOrQuit));
                    }
                }
                Action::CopyUnstyledOutput => {
                    mission_state.copy_unstyled_output();
                }
                Action::DismissTop => {
                    mission_state.dismiss_top();
                }
                Action::DismissTopItem => {
                    mission_state.dismiss_top_item();
                }
                Action::DismissTopItemType => {
                    if !mission_state.dismiss_top_item_type() {
                        mission_state
                            .messages
                            .push(Message::short("No type found for the top item"));
                    }
                }
                Action::UndismissAll => {
                    mission_state.undismiss_all();
                    rerun = true;
                }
                Action::UndismissLocation(location) => {
                    mission_state.remove_dismissal(&Dismissal::Location(location));
                    rerun = true;
                }
                Action::UndismissDiagType(diag_type) => {
                    mission_state.remove_dismissal(&Dismissal::DiagType(diag_type));
                    rerun = true;
                }
                Action::OpenUndismissMenu => {
                    mission_state.open_undismiss_menu();
                }
                Action::Export(export_name) => {
                    let export_name = export_name.clone();
                    mission_state
                        .mission
                        .settings
                        .exports
                        .do_named_export(&export_name, &mission_state);
                    mission_state
                        .messages
                        .push(Message::short(format!("Export *{}* done", &export_name)));
                }
                Action::FocusFile(focus_file_command) => {
                    mission_state.focus_file(&focus_file_command);
                }
                Action::FocusGoto => {
                    mission_state.focus_goto();
                }
                Action::FocusSearch => {
                    mission_state.focus_search();
                }
                Action::Help => {
                    mission_state.toggle_help();
                }
                Action::Job(job_ref) => {
                    mission_end = Some(job_ref.clone().into());
                    break;
                }
                Action::NextMatch => {
                    mission_state.next_match();
                }
                Action::NoOp => {}
                Action::OpenJobsMenu => {
                    mission_state.open_jobs_menu();
                }
                Action::OpenMenu(definition) => {
                    mission_state.open_menu(*definition);
                }
                Action::Pause => {
                    mission_state.auto_refresh = AutoRefresh::Paused;
                }
                Action::PlaySound(play_sound_command) => {
                    if let Some(sound_player) = &sound_player {
                        sound_player.play(play_sound_command.clone());
                    } else if !sound_not_enabled_message_already_displayed {
                        let message = {
                            #[cfg(not(feature = "sound"))]
                            {
                                "Sound requested but not enabled in this build"
                            }
                            #[cfg(feature = "sound")]
                            {
                                "Sound requested but not enabled in config"
                            }
                        };
                        debug!("{message}");
                        sound_not_enabled_message_already_displayed = true;
                        mission_state.messages.push(Message::short(message));
                    }
                }
                Action::PreviousMatch => {
                    mission_state.previous_match();
                }
                Action::Quit => {
                    mission_end = Some(DoAfterMission::Quit);
                    break;
                }
                Action::ReRun => {
                    rerun = true;
                }
                Action::Refresh => {
                    mission_state.clear();
                    rerun = true;
                }
                Action::ReloadConfig => {
                    mission_end = Some(DoAfterMission::ReloadConfig);
                    break;
                }
                Action::ScopeToFailures => {
                    if let Some(scope) = mission_state.failures_scope() {
                        info!("scoping to failures: {scope:#?}");
                        mission_end = Some(JobRef::from(scope).into());
                        break;
                    }
                    warn!("no available failures scope");
                }
                Action::Scroll(scroll_command) => {
                    mission_state.apply_scroll_command(scroll_command);
                }
                Action::ShowItem(show_item_command) => {
                    mission_state.show_item(show_item_command.item_idx);
                }
                Action::ToggleBacktrace(level) => {
                    mission_state.toggle_backtrace(level);
                    rerun = true;
                }
                Action::TogglePause => match mission_state.auto_refresh {
                    AutoRefresh::Enabled => {
                        mission_state.auto_refresh = AutoRefresh::Paused;
                    }
                    AutoRefresh::Paused => {
                        mission_state.auto_refresh = AutoRefresh::Enabled;
                        if mission_state.changes_since_last_job_start > 0 {
                            rerun = true;
                        }
                    }
                },
                Action::ToggleRawOutput => {
                    mission_state.toggle_raw_output();
                }
                Action::ToggleSummary => {
                    mission_state.toggle_summary_mode();
                }
                Action::ToggleWrap => {
                    mission_state.toggle_wrap_mode();
                }
                Action::Unpause => {
                    if mission_state.changes_since_last_job_start > 0 {
                        rerun = true;
                    } else {
                        mission_state.auto_refresh = AutoRefresh::Enabled;
                    }
                }
                Action::Validate => {
                    mission_state.validate();
                }
            }
            if rerun {
                task_executor.die();
                task_executor = mission_state.start_computation(&mut executor)?;
                break; // drop following actions
            }
        }
        if !headless {
            mission_state.draw(w)?;
        }
        if let Some(mission_end) = mission_end {
            task_executor.die();
            return Ok(mission_end);
        }
    }
}
