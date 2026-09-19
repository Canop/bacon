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
    rustc_hash::FxHashSet,
    std::path::{
        Path,
        PathBuf,
    },
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

/// Tells whether a path received in a notify event is one we watch.
///
/// Watched directories are watched recursively, so any path below them is
/// relevant. Watched files are watched through their parent directory, whose
/// other entries must be filtered out.
#[derive(Debug, Default)]
struct Filter {
    dirs: Vec<PathBuf>,
    files: FxHashSet<PathBuf>,
}

impl Filter {
    fn add_dir(
        &mut self,
        path: &Path,
    ) {
        self.dirs.push(path.to_path_buf());
        if let Some(canonic) = canonic(path) {
            self.dirs.push(canonic);
        }
    }
    fn add_file(
        &mut self,
        path: &Path,
    ) {
        self.files.insert(path.to_path_buf());
        if let Some(canonic) = canonic(path) {
            self.files.insert(canonic);
        }
    }
    /// Tell whether the path is in a directory watched recursively
    fn is_in_watched_dir(
        &self,
        path: &Path,
    ) -> bool {
        self.dirs.iter().any(|dir| path.starts_with(dir))
    }
    fn is_relevant(
        &self,
        path: &Path,
    ) -> bool {
        self.files.contains(path) || self.is_in_watched_dir(path)
    }
}

/// The canonical form of the path, when it differs from the path itself.
///
/// Events may come with canonicalized paths (that's the case on macOS) while
/// the paths to watch may go through symlinks, so both forms must be known.
/// The path may not exist yet, in which case its parent is canonicalized.
fn canonic(path: &Path) -> Option<PathBuf> {
    let canonic = match path.canonicalize() {
        Ok(canonic) => canonic,
        Err(_) => path.parent()?.canonicalize().ok()?.join(path.file_name()?),
    };
    (canonic != path).then_some(canonic)
}

/// A directory watched non recursively, for some of the files it contains
#[derive(Debug)]
struct WatchDir {
    path: PathBuf,
    /// canonical form, so that two spellings of the same directory
    /// (eg one going through a symlink) aren't watched twice
    canonic: PathBuf,
}

impl WatchDir {
    fn new(path: &Path) -> Self {
        Self {
            canonic: canonic(path).unwrap_or_else(|| path.to_path_buf()),
            path: path.to_path_buf(),
        }
    }
}

/// How to watch the paths of a mission: what to give to notify, and how to
/// filter the events it sends back
#[derive(Debug, Default)]
struct WatchPlan {
    recursive: Vec<PathBuf>,
    non_recursive: Vec<WatchDir>,
    filter: Filter,
}

impl WatchPlan {
    fn add_path(
        &mut self,
        path: &Path,
    ) {
        // notify builds event paths from the paths it's given, so we work on
        // absolute ones to be able to recognize them
        let absolute;
        let path = if path.is_absolute() {
            path
        } else {
            let Ok(current_dir) = std::env::current_dir() else {
                warn!("can't resolve relative path {path:?}");
                return;
            };
            absolute = current_dir.join(path);
            &absolute
        };
        if path.is_dir() {
            self.filter.add_dir(path);
            // a recursive watch supersedes the non recursive ones below it,
            // whatever their spelling
            let canonic = canonic(path);
            let canonic = canonic.as_deref().unwrap_or(path);
            self.non_recursive
                .retain(|dir| !dir.path.starts_with(path) && !dir.canonic.starts_with(canonic));
            debug!("add watch dir {path:?}");
            self.recursive.push(path.to_path_buf());
            return;
        }
        self.filter.add_file(path);
        self.watch_parent_of(path);
        // a watched file may be a symlink, whose target is in another directory
        if let Some(canonic) = canonic(path) {
            self.watch_parent_of(&canonic);
        }
    }
    /// Watch the directory containing the file instead of the file itself:
    /// inotify and kqueue bind a watch to the inode, so a watch on a file is
    /// lost as soon as an editor saves by replacing the file instead of
    /// writing in place. The other entries of the directory are filtered out.
    fn watch_parent_of(
        &mut self,
        file: &Path,
    ) {
        let Some(parent) = file.parent() else {
            warn!("no directory to watch for {file:?}");
            return;
        };
        let dir = WatchDir::new(parent);
        if self.filter.is_in_watched_dir(&dir.path) || self.filter.is_in_watched_dir(&dir.canonic) {
            return; // already covered by a recursive watch
        }
        if !parent.is_dir() {
            warn!("watch path doesn't exist: {file:?}");
            return;
        }
        if self.non_recursive.iter().any(|d| d.canonic == dir.canonic) {
            return;
        }
        debug!("add watch dir {parent:?} for file {file:?}");
        self.non_recursive.push(dir);
    }
}

/// Tell whether a notify event is worth notifying the mission about
fn should_notify(
    event: &notify::Event,
    filter: &Filter,
    ignorer: &mut IgnorerSet,
) -> bool {
    if event.need_rescan() {
        // the system dropped events: we can't know what changed
        info!("events dropped by the system: {event:?}");
        return true;
    }
    match event.kind {
        EventKind::Modify(ModifyKind::Metadata(_)) => {
            //debug!("ignoring metadata change");
            return false; // useless event
        }
        EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
            //debug!("ignoring 'any' data change");
            return false; // probably useless event with no real change
        }
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
            debug!("close write event: {event:?}");
        }
        EventKind::Access(_) => {
            //debug!("ignoring access event: {event:?}");
            return false; // probably useless event
        }
        _ => {
            info!("notify event: {event:?}");
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
    true
}

impl Watcher {
    pub fn new(
        paths_to_watch: &[PathBuf],
        mut ignorer: IgnorerSet,
    ) -> Result<Self> {
        info!("watcher on {paths_to_watch:#?}");
        let mut plan = WatchPlan::default();
        for path in paths_to_watch {
            plan.add_path(path);
        }
        let WatchPlan {
            recursive,
            non_recursive,
            filter,
        } = plan;
        let (sender, receiver) = bounded(0);
        let mut notify_watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                Ok(we) => {
                    if should_notify(&we, &filter, &mut ignorer) {
                        if let Err(e) = sender.send(()) {
                            debug!("error when notifying on notify event: {e}");
                        }
                    }
                }
                Err(e) => warn!("watch error: {e:?}"),
            })?;
        for path in recursive {
            notify_watcher.watch(&path, RecursiveMode::Recursive)?;
        }
        for dir in non_recursive {
            notify_watcher.watch(&dir.path, RecursiveMode::NonRecursive)?;
        }
        Ok(Self {
            receiver,
            _notify_watcher: notify_watcher,
        })
    }
}

#[cfg(test)]
mod watcher_tests {
    use {
        super::*,
        notify::{
            Event,
            event::Flag,
        },
        std::{
            fs,
            sync::atomic::{
                AtomicUsize,
                Ordering,
            },
            time::Duration,
        },
    };

    /// A directory in the system temp dir, removed on drop
    struct TestDir {
        path: PathBuf,
    }
    impl TestDir {
        fn new() -> Self {
            static COUNTER: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "bacon-watcher-test-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
        fn path(
            &self,
            relative: &str,
        ) -> PathBuf {
            self.path.join(relative)
        }
        /// Write the file the way many editors do: in a new file which is then
        /// renamed over the target, which changes the inode of the target
        fn save_by_replacement(
            &self,
            relative: &str,
            content: &str,
        ) {
            let target = self.path(relative);
            let temp = self.path(&format!("{relative}.tmp"));
            fs::write(&temp, content).unwrap();
            fs::rename(&temp, &target).unwrap();
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    /// Receive until nothing comes for a short while
    fn drain(watcher: &Watcher) {
        while watcher
            .receiver
            .recv_timeout(Duration::from_millis(500))
            .is_ok()
        {}
    }

    fn plan_of(paths: &[PathBuf]) -> WatchPlan {
        let mut plan = WatchPlan::default();
        for path in paths {
            plan.add_path(path);
        }
        plan
    }

    /// The directories watched non recursively, sorted
    fn non_recursive_paths(plan: &WatchPlan) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = plan.non_recursive.iter().map(|d| d.path.clone()).collect();
        paths.sort();
        paths
    }

    /// A file is watched through its parent directory, and a directory
    /// already watched recursively isn't watched twice
    #[test]
    fn test_watch_plan() {
        let dir = TestDir::new();
        fs::create_dir_all(dir.path("src")).unwrap();
        fs::write(dir.path("Cargo.toml"), "").unwrap();
        fs::write(dir.path("src/main.rs"), "").unwrap();
        let paths = vec![
            dir.path("src"),
            dir.path("Cargo.toml"),
            dir.path("src/main.rs"),
            dir.path("build.rs"), // doesn't exist
        ];
        let mut plan = plan_of(&paths);
        assert_eq!(plan.recursive, vec![dir.path("src")]);
        assert_eq!(non_recursive_paths(&plan), vec![dir.path.clone()]);
        assert!(plan.filter.is_relevant(&dir.path("src/main.rs")));
        assert!(plan.filter.is_relevant(&dir.path("src/deep/mod.rs")));
        assert!(plan.filter.is_relevant(&dir.path("Cargo.toml")));
        assert!(plan.filter.is_relevant(&dir.path("build.rs")));
        assert!(!plan.filter.is_relevant(&dir.path("README.md")));
        assert!(!plan.filter.is_relevant(&dir.path("target/debug/bacon")));

        // the plan doesn't depend on the order of the paths
        let mut reversed = paths.clone();
        reversed.reverse();
        let mut reversed_plan = plan_of(&reversed);
        reversed_plan.recursive.sort();
        plan.recursive.sort();
        assert_eq!(plan.recursive, reversed_plan.recursive);
        assert_eq!(
            non_recursive_paths(&plan),
            non_recursive_paths(&reversed_plan)
        );
        assert_eq!(plan.filter.files, reversed_plan.filter.files);
    }

    /// Check that a file is still watched after having been saved by
    /// replacement, which changes its inode
    #[test]
    fn test_watch_file_saved_by_replacement() {
        let dir = TestDir::new();
        fs::write(dir.path("build.rs"), "fn main() {}").unwrap();
        let watcher = Watcher::new(&[dir.path("build.rs")], IgnorerSet::default()).unwrap();
        dir.save_by_replacement("build.rs", "fn main() { // 1 }");
        assert!(
            watcher.receiver.recv_timeout(Duration::from_secs(5)).is_ok(),
            "no notification on first save",
        );
        drain(&watcher);
        dir.save_by_replacement("build.rs", "fn main() { // 2 }");
        assert!(
            watcher.receiver.recv_timeout(Duration::from_secs(5)).is_ok(),
            "no notification on second save: the file isn't watched anymore",
        );
    }

    /// Check that watching a file doesn't make its siblings watched
    #[test]
    fn test_sibling_of_watched_file_isnt_watched() {
        let dir = TestDir::new();
        fs::write(dir.path("Cargo.toml"), "").unwrap();
        let watcher = Watcher::new(&[dir.path("Cargo.toml")], IgnorerSet::default()).unwrap();
        dir.save_by_replacement("README.md", "not a watched file");
        assert!(
            watcher
                .receiver
                .recv_timeout(Duration::from_secs(1))
                .is_err(),
            "notification on a file which isn't watched",
        );
    }

    /// Check that a watched file which is a symlink is watched through its
    /// target too, as that's where the changes happen
    #[cfg(unix)]
    #[test]
    fn test_watch_symlinked_file() {
        let dir = TestDir::new();
        fs::create_dir_all(dir.path("elsewhere")).unwrap();
        fs::create_dir_all(dir.path("project")).unwrap();
        let target = dir.path("elsewhere/bacon.toml");
        let link = dir.path("project/bacon.toml");
        fs::write(&target, "a").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let watcher = Watcher::new(&[link], IgnorerSet::default()).unwrap();
        dir.save_by_replacement("elsewhere/bacon.toml", "b");
        assert!(
            watcher.receiver.recv_timeout(Duration::from_secs(5)).is_ok(),
            "no notification on a change of the symlink target",
        );
    }

    /// Check that a directory reached through a symlink isn't watched a
    /// second time when it's already in a recursively watched directory
    #[cfg(unix)]
    #[test]
    fn test_aliased_dir_isnt_watched_twice() {
        let dir = TestDir::new();
        fs::create_dir_all(dir.path("p/sub")).unwrap();
        fs::write(dir.path("p/sub/f.rs"), "").unwrap();
        std::os::unix::fs::symlink(dir.path("p/sub"), dir.path("q")).unwrap();
        let paths = vec![dir.path("p"), dir.path("q/f.rs")];
        assert!(non_recursive_paths(&plan_of(&paths)).is_empty());
        let mut reversed = paths.clone();
        reversed.reverse();
        assert!(non_recursive_paths(&plan_of(&reversed)).is_empty());
    }

    /// Check that a relative path is watched (cargo runs tests in the
    /// directory of the crate, which holds a Cargo.toml)
    #[test]
    fn test_watch_relative_path() {
        let current_dir = std::env::current_dir().unwrap();
        let plan = plan_of(&[PathBuf::from("Cargo.toml")]);
        assert_eq!(non_recursive_paths(&plan), vec![current_dir.clone()]);
        assert!(plan.filter.is_relevant(&current_dir.join("Cargo.toml")));
        // a relative path to a file which doesn't exist yet too
        let plan = plan_of(&[PathBuf::from("not-yet.rs")]);
        assert_eq!(non_recursive_paths(&plan), vec![current_dir.clone()]);
        assert!(plan.filter.is_relevant(&current_dir.join("not-yet.rs")));
    }

    /// Check that an event telling events were dropped is transmitted:
    /// we can't know what changed, so we must assume the worst
    #[test]
    fn test_dropped_events_notify() {
        let dir = TestDir::new();
        let mut filter = Filter::default();
        filter.add_file(&dir.path("Cargo.toml"));
        let mut ignorer = IgnorerSet::default();
        let dropped = Event::new(EventKind::Other).set_flag(Flag::Rescan);
        assert!(should_notify(&dropped, &filter, &mut ignorer));
        // on macOS the event comes with the path of the watched directory,
        // which isn't a watched file
        let dropped = Event::new(EventKind::Other)
            .set_flag(Flag::Rescan)
            .add_path(dir.path.clone());
        assert!(should_notify(&dropped, &filter, &mut ignorer));
    }

    /// Check the filtering done on the events which carry paths
    #[test]
    fn test_should_notify_filters_on_path() {
        let dir = TestDir::new();
        let mut filter = Filter::default();
        filter.add_file(&dir.path("Cargo.toml"));
        let mut ignorer = IgnorerSet::default();
        let event = |relative: &str| {
            Event::new(EventKind::Modify(ModifyKind::Any)).add_path(dir.path(relative))
        };
        assert!(should_notify(&event("Cargo.toml"), &filter, &mut ignorer));
        assert!(!should_notify(&event("README.md"), &filter, &mut ignorer));
        let pathless = Event::new(EventKind::Modify(ModifyKind::Any));
        assert!(!should_notify(&pathless, &filter, &mut ignorer));
    }
}
