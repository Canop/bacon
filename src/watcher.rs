use {
    crate::*,
    anyhow::Result,
    notify::{
        event::{
            AccessKind, AccessMode, DataChange, EventKind, MetadataKind, ModifyKind
        }, RecursiveMode, Watcher as NotifyWatcher
    },
    std::{path::PathBuf, time::Duration},
    termimad::crossbeam::channel::{
        bounded, Receiver
    },
};

/// A file watcher, providing a channel to receive notifications
pub struct Watcher {
    pub receiver: Receiver<()>,
    _notify_watcher: Box<dyn NotifyWatcher>,
}

impl Watcher {
    pub fn new(
        paths_to_watch: &[PathBuf],
        mut ignorer: IgnorerSet,
        polling: Option<u64>,
    ) -> Result<Self> {
        info!("watcher on {:#?}", paths_to_watch);
        let (sender, receiver) = bounded(0);
        let event_handler = move |res: notify::Result<notify::Event>| match res {
                Ok(we) => {
                    match we.kind {
                        EventKind::Modify(ModifyKind::Metadata(kind)) => {
                            if kind == MetadataKind::WriteTime && polling.is_some() {
                                // the only event triggered usable when polling on WSL2   
                            } else {
                                debug!("ignoring metadata change: {:?}", kind);
                                return; // useless event
                            }
                        }
                        EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                            debug!("ignoring 'any' data change");
                            return; // probably useless event with no real change
                        }
                        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                            info!("close write event: {we:?}");
                        }
                        EventKind::Access(_) => {
                            debug!("ignoring access event: {we:?}");
                            return; // probably useless event
                        }
                        _ => {
                            info!("notify event: {we:?}");
                        }
                    }
                    match time!(Info, ignorer.excludes_all_pathbufs(&we.paths)) {
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
                    if let Err(e) = sender.send(()) {
                        debug!("error when notifying on notify event: {}", e);
                    }
                }
                Err(e) => warn!("watch error: {:?}", e),
            };
            let use_polling = polling.is_some();
        let notify_watcher: Result<Box<dyn NotifyWatcher>, notify::Error> = if use_polling {
            let config = notify::Config::default()
                .with_poll_interval(Duration::from_secs(polling.unwrap_or_default()))
                .with_compare_contents(true);
            notify::PollWatcher::new(event_handler, config)
                .map(|w| Box::new(w) as Box<dyn NotifyWatcher>)
        } else {
            notify::recommended_watcher(event_handler)
                .map(|w| Box::new(w) as Box<dyn NotifyWatcher>)
        };
        let mut notify_watcher = notify_watcher?;

        for path in paths_to_watch {
            if !path.exists() {
                warn!("watch path doesn't exist: {:?}", path);
                continue;
            }
            if path.is_dir() {
                debug!("add watch dir {:?}", path);
                notify_watcher.watch(path, RecursiveMode::Recursive)?;
            } else if path.is_file() {
                debug!("add watch file {:?}", path);
                notify_watcher.watch(path, RecursiveMode::NonRecursive)?;
            }
        }
        Ok(Self {
            receiver,
            _notify_watcher: notify_watcher,
        })
    }
}
