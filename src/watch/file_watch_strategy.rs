use {
    notify::{
        RecommendedWatcher,
        Watcher as NotifyWatcher,
        WatcherKind,
    },
    schemars::JsonSchema,
    serde::Deserialize,
};

/// How a file to watch (as opposed to a directory, which is always
/// watched recursively) is given to notify.
///
/// The default depends on the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FileWatchStrategy {
    /// Watch the directory containing the file, and filter out the
    /// events regarding its other entries. Needed when the backend
    /// binds a watch to an inode: the watch would be lost as soon as
    /// an editor saves by replacing the file instead of writing in place.
    ParentDir,
    /// Watch the file itself, the backend following the path.
    File,
    /// Watch the file by polling it, which follows the path. For a
    /// backend binding a watch to an inode and unable to tell which
    /// entry of a directory changed (kqueue).
    FilePoll,
}

impl FileWatchStrategy {
    pub fn default_for(kind: WatcherKind) -> Self {
        match kind {
            WatcherKind::Inotify => Self::ParentDir,
            WatcherKind::Kqueue => Self::FilePoll,
            _ => Self::File,
        }
    }
}

impl Default for FileWatchStrategy {
    fn default() -> Self {
        Self::default_for(RecommendedWatcher::kind())
    }
}
