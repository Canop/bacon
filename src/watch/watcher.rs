use {
    super::{
        FileWatchStrategy,
        plan::{
            Filter,
            WatchPlan,
        },
    },
    crate::*,
    anyhow::Result,
    notify::{
        Config,
        PollWatcher,
        RecommendedWatcher,
        RecursiveMode,
        Watcher as NotifyWatcher,
        WatcherKind,
        event::{
            AccessKind,
            AccessMode,
            DataChange,
            EventKind,
            ModifyKind,
        },
    },
    std::{
        path::PathBuf,
        sync::{
            Arc,
            Mutex,
        },
        time::Duration,
    },
    termimad::crossbeam::channel::{
        Receiver,
        Sender,
        TrySendError,
        bounded,
    },
};

/// A file watcher, providing a channel to receive notifications.
///
/// The channel holds at most one pending notification: a change is worth
/// notifying only once until it has been received.
pub struct Watcher {
    pub receiver: Receiver<()>,
    _notify_watcher: RecommendedWatcher,
    /// watcher of the files, when they're polled
    _poll_watcher: Option<PollWatcher>,
}

/// Tell whether a notify event is worth notifying the mission about
pub(super) fn should_notify(
    event: &notify::Event,
    kind: WatcherKind,
    filter: &Filter,
    ignorer: &mut IgnorerSet,
) -> bool {
    if event.need_rescan() {
        // events were dropped, we can't know what changed
        info!("rescan needed: {event:?}");
        return true;
    }
    // the poll watcher only reports real changes, the other ones
    // send many events of no interest
    if kind != WatcherKind::PollWatcher {
        match event.kind {
            EventKind::Modify(ModifyKind::Metadata(_)) => {
                //debug!("ignoring metadata change");
                return false; // useless event
            }
            EventKind::Modify(ModifyKind::Data(DataChange::Any)) if kind != WatcherKind::Kqueue => {
                // probably useless event with no real change (but it's the only
                // form of a write on kqueue)
                //debug!("ignoring 'any' data change");
                return false;
            }
            EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                debug!("close write event: {event:?}");
            }
            EventKind::Access(_) => {
                //debug!("ignoring access event: {event:?}");
                return false; // probably useless event
            }
            _ => {
                debug!("notify event: {event:?}");
            }
        }
    }
    let paths: Vec<PathBuf> = event
        .paths
        .iter()
        .filter(|path| filter.is_relevant(path))
        .cloned()
        .collect();
    if paths.is_empty() {
        debug!("no watched path in event");
        return false;
    }
    match time!(Info, ignorer.excludes_all_pathbufs(&paths)) {
        Ok(true) => {
            debug!("all excluded");
            return false;
        }
        Ok(false) => {
            debug!("at least one is included");
        }
        Err(e) => {
            warn!("exclusion check failed: {e}");
        }
    }
    info!("change in watched path(s): {paths:?}");
    true
}

/// What receives the events of the notify watchers, and decides
/// whether to notify the mission
struct Handler {
    sender: Sender<()>,
    filter: Filter,
    ignorer: IgnorerSet,
}

impl Handler {
    fn handle(
        &mut self,
        res: notify::Result<notify::Event>,
        kind: WatcherKind,
    ) {
        let Ok(event) = res.inspect_err(|e| warn!("watch error: {e:?}")) else {
            return;
        };
        if self.sender.is_full() {
            return; // a notification is already pending
        }
        if should_notify(&event, kind, &self.filter, &mut self.ignorer) {
            if let Err(TrySendError::Disconnected(_)) = self.sender.try_send(()) {
                debug!("watch receiver disconnected");
            }
        }
    }
}

impl Watcher {
    pub fn new(
        paths_to_watch: &[PathBuf],
        ignorer: IgnorerSet,
        strategy: FileWatchStrategy,
    ) -> Result<Self> {
        info!("watcher on {paths_to_watch:#?}");
        let mut plan = WatchPlan::new(strategy);
        for path in paths_to_watch {
            plan.add_path(path);
        }
        let WatchPlan {
            recursive,
            non_recursive,
            files,
            filter,
            ..
        } = plan;
        let (sender, receiver) = bounded(1);
        let handler = Arc::new(Mutex::new(Handler {
            sender,
            filter,
            ignorer,
        }));
        let mut notify_watcher = {
            let handler = Arc::clone(&handler);
            let kind = RecommendedWatcher::kind();
            notify::recommended_watcher(move |res| handler.lock().unwrap().handle(res, kind))?
        };
        let mut poll_watcher = match strategy {
            FileWatchStrategy::FilePoll if !files.is_empty() => {
                let handler = Arc::clone(&handler);
                let config = Config::default().with_poll_interval(Duration::from_secs(1));
                Some(PollWatcher::new(
                    move |res| handler.lock().unwrap().handle(res, WatcherKind::PollWatcher),
                    config,
                )?)
            }
            _ => None,
        };
        // paths are added in one batch: on FSEvents, each `watch` call
        // would stop and restart the stream and its thread
        let mut paths = notify_watcher.paths_mut();
        for path in &recursive {
            paths.add(path, RecursiveMode::Recursive)?;
        }
        for dir in &non_recursive {
            paths.add(&dir.path, RecursiveMode::NonRecursive)?;
        }
        if let Some(poll_watcher) = &mut poll_watcher {
            for file in &files {
                poll_watcher.watch(&file.path, RecursiveMode::NonRecursive)?;
            }
        } else {
            for file in &files {
                paths.add(&file.path, RecursiveMode::NonRecursive)?;
            }
        }
        paths.commit()?;
        Ok(Self {
            receiver,
            _notify_watcher: notify_watcher,
            _poll_watcher: poll_watcher,
        })
    }
}
