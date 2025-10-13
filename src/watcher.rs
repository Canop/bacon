use {
    crate::*,
    anyhow::Result,
    notify::{
        RecommendedWatcher,
        RecursiveMode,
        Watcher as NotifyWatcher,
        event::{
            AccessKind,
            AccessMode,
            DataChange,
            EventKind,
            ModifyKind,
        },
    },
    std::path::PathBuf,
    termimad::crossbeam::channel::{
        Receiver,
        bounded,
    },
};

/// A file watcher, providing a channel to receive notifications
pub struct Watcher {
    pub receiver: Receiver<()>,
    _notify_watcher: RecommendedWatcher,
}

impl Watcher {
    pub fn new(
        paths_to_watch: &[PathBuf],
        mut ignorer: IgnorerSet,
    ) -> Result<Self> {
        info!("watcher on {:#?}", paths_to_watch);
        let (sender, receiver) = bounded(0);
        let mut notify_watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                Ok(we) => {
                    match we.kind {
                        EventKind::Modify(ModifyKind::Metadata(_)) => {
                            //debug!("ignoring metadata change");
                            return; // useless event
                        }
                        EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                            //debug!("ignoring 'any' data change");
                            return; // probably useless event with no real change
                        }
                        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                            debug!("close write event: {we:?}");
                        }
                        EventKind::Access(_) => {
                            //debug!("ignoring access event: {we:?}");
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
            })?;
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
