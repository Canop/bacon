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
    _notify_watcher: RecommendedWatcher,
    pub receiver: Receiver<()>,
}

impl Watcher {
    pub fn new(
        files_to_watch: &[PathBuf],
        directories_to_watch: &[PathBuf],
        mut ignorer: Option<Ignorer>,
    ) -> Result<Self> {
        let (sender, receiver) = bounded(0);
        let mut notify_watcher =
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
                    if let Err(e) = sender.send(()) {
                        debug!("error when notifying on notify event: {}", e);
                    }
                }
                Err(e) => warn!("watch error: {:?}", e),
            })?;
        for file in files_to_watch {
            debug!("add watch file {:?}", file);
            notify_watcher.watch(file, RecursiveMode::NonRecursive)?;
        }
        for dir in directories_to_watch {
            debug!("add watch dir {:?}", dir);
            notify_watcher.watch(dir, RecursiveMode::Recursive)?;
        }
        Ok(Self {
            _notify_watcher: notify_watcher,
            receiver,
        })
    }
}
