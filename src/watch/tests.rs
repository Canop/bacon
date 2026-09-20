use {
    super::{
        FileWatchStrategy,
        Watcher,
        plan::{
            Filter,
            WatchPlan,
        },
        watcher::should_notify,
    },
    crate::IgnorerSet,
    notify::{
        Event,
        WatcherKind,
        event::{
            DataChange,
            EventKind,
            Flag,
            MetadataKind,
            ModifyKind,
        },
    },
    std::{
        fs,
        path::PathBuf,
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

fn plan_of(
    strategy: FileWatchStrategy,
    paths: &[PathBuf],
) -> WatchPlan {
    let mut plan = WatchPlan::new(strategy);
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

/// The files watched directly, sorted
fn file_paths(plan: &WatchPlan) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = plan.files.iter().map(|f| f.path.clone()).collect();
    paths.sort();
    paths
}

fn watcher_on(paths: &[PathBuf]) -> Watcher {
    watcher_with(FileWatchStrategy::default(), paths)
}

fn watcher_with(
    strategy: FileWatchStrategy,
    paths: &[PathBuf],
) -> Watcher {
    Watcher::new(paths, IgnorerSet::default(), strategy).unwrap()
}

/// With the parent dir strategy, a file is watched through its parent
/// directory, and a directory already watched recursively isn't watched twice
#[test]
fn test_watch_plan_parent_dir() {
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
    let mut plan = plan_of(FileWatchStrategy::ParentDir, &paths);
    assert_eq!(plan.recursive, vec![dir.path("src")]);
    assert_eq!(non_recursive_paths(&plan), vec![dir.path.clone()]);
    assert!(file_paths(&plan).is_empty());
    assert!(plan.filter.is_relevant(&dir.path("src/main.rs")));
    assert!(plan.filter.is_relevant(&dir.path("src/deep/mod.rs")));
    assert!(plan.filter.is_relevant(&dir.path("Cargo.toml")));
    assert!(plan.filter.is_relevant(&dir.path("build.rs")));
    assert!(!plan.filter.is_relevant(&dir.path("README.md")));
    assert!(!plan.filter.is_relevant(&dir.path("target/debug/bacon")));

    // the plan doesn't depend on the order of the paths
    let mut reversed = paths.clone();
    reversed.reverse();
    let mut reversed_plan = plan_of(FileWatchStrategy::ParentDir, &reversed);
    reversed_plan.recursive.sort();
    plan.recursive.sort();
    assert_eq!(plan.recursive, reversed_plan.recursive);
    assert_eq!(
        non_recursive_paths(&plan),
        non_recursive_paths(&reversed_plan)
    );
    assert_eq!(plan.filter.files, reversed_plan.filter.files);
}

/// With the file strategies, a file is watched directly, unless it's
/// in a directory watched recursively, and a missing file isn't watched
#[test]
fn test_watch_plan_file() {
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
    for strategy in [FileWatchStrategy::File, FileWatchStrategy::FilePoll] {
        let plan = plan_of(strategy, &paths);
        assert_eq!(plan.recursive, vec![dir.path("src")]);
        assert!(non_recursive_paths(&plan).is_empty());
        assert_eq!(file_paths(&plan), vec![dir.path("Cargo.toml")]);
        assert!(plan.filter.is_relevant(&dir.path("Cargo.toml")));
        assert!(plan.filter.is_relevant(&dir.path("src/deep/mod.rs")));
        assert!(!plan.filter.is_relevant(&dir.path("README.md")));

        // the plan doesn't depend on the order of the paths
        let mut reversed = paths.clone();
        reversed.reverse();
        let reversed_plan = plan_of(strategy, &reversed);
        assert_eq!(plan.recursive, reversed_plan.recursive);
        assert_eq!(file_paths(&plan), file_paths(&reversed_plan));
        assert_eq!(plan.filter.files, reversed_plan.filter.files);
    }
}

/// Check that a file is still watched after having been saved by
/// replacement, which changes its inode
#[test]
fn test_watch_file_saved_by_replacement() {
    let dir = TestDir::new();
    fs::write(dir.path("build.rs"), "fn main() {}").unwrap();
    let watcher = watcher_on(&[dir.path("build.rs")]);
    dir.save_by_replacement("build.rs", "fn main() { // 1 }");
    assert!(
        watcher
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .is_ok(),
        "no notification on first save",
    );
    drain(&watcher);
    dir.save_by_replacement("build.rs", "fn main() { // 2 }");
    assert!(
        watcher
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .is_ok(),
        "no notification on second save: the file isn't watched anymore",
    );
}

/// Same check, with the file polled, whatever the platform
#[test]
fn test_polled_file_saved_by_replacement() {
    let dir = TestDir::new();
    fs::write(dir.path("build.rs"), "fn main() {}").unwrap();
    let watcher = watcher_with(FileWatchStrategy::FilePoll, &[dir.path("build.rs")]);
    for round in 1..=2 {
        std::thread::sleep(Duration::from_millis(1100)); // ensure a new mtime
        dir.save_by_replacement("build.rs", &format!("fn main() {{ // {round} }}"));
        assert!(
            watcher
                .receiver
                .recv_timeout(Duration::from_secs(5))
                .is_ok(),
            "no notification on save {round}",
        );
    }
}

/// Check that watching a file doesn't make its siblings watched
#[test]
fn test_sibling_of_watched_file_isnt_watched() {
    let dir = TestDir::new();
    fs::write(dir.path("Cargo.toml"), "").unwrap();
    let watcher = watcher_on(&[dir.path("Cargo.toml")]);
    dir.save_by_replacement("README.md", "not a watched file");
    assert!(
        watcher
            .receiver
            .recv_timeout(Duration::from_secs(1))
            .is_err(),
        "notification on a file which isn't watched",
    );
}

/// With the file strategies, a symlink is watched with its target,
/// and a plain file is watched as given
#[cfg(unix)]
#[test]
fn test_watch_plan_symlinked_file() {
    let dir = TestDir::new();
    fs::create_dir_all(dir.path("elsewhere")).unwrap();
    fs::create_dir_all(dir.path("project")).unwrap();
    let target = dir.path("elsewhere/bacon.toml");
    let link = dir.path("project/bacon.toml");
    fs::write(&target, "").unwrap();
    fs::write(dir.path("project/Cargo.toml"), "").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let plan = plan_of(
        FileWatchStrategy::File,
        &[link.clone(), dir.path("project/Cargo.toml")],
    );
    let mut expected = vec![
        link,
        target.canonicalize().unwrap(),
        dir.path("project/Cargo.toml"),
    ];
    expected.sort();
    assert_eq!(file_paths(&plan), expected);
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
    let watcher = watcher_on(&[link]);
    dir.save_by_replacement("elsewhere/bacon.toml", "b");
    assert!(
        watcher
            .receiver
            .recv_timeout(Duration::from_secs(5))
            .is_ok(),
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
    let mut reversed = paths.clone();
    reversed.reverse();
    for strategy in [
        FileWatchStrategy::ParentDir,
        FileWatchStrategy::File,
        FileWatchStrategy::FilePoll,
    ] {
        for paths in [&paths, &reversed] {
            let plan = plan_of(strategy, paths);
            assert!(non_recursive_paths(&plan).is_empty());
            assert!(file_paths(&plan).is_empty());
        }
    }
}

/// Check that a relative path is watched (cargo runs tests in the
/// directory of the crate, which holds a Cargo.toml)
#[test]
fn test_watch_relative_path() {
    let current_dir = std::env::current_dir().unwrap();
    let plan = plan_of(FileWatchStrategy::ParentDir, &[PathBuf::from("Cargo.toml")]);
    assert_eq!(non_recursive_paths(&plan), vec![current_dir.clone()]);
    assert!(plan.filter.is_relevant(&current_dir.join("Cargo.toml")));
    // a relative path to a file which doesn't exist yet too
    let plan = plan_of(FileWatchStrategy::ParentDir, &[PathBuf::from("not-yet.rs")]);
    assert_eq!(non_recursive_paths(&plan), vec![current_dir.clone()]);
    assert!(plan.filter.is_relevant(&current_dir.join("not-yet.rs")));
    let plan = plan_of(FileWatchStrategy::File, &[PathBuf::from("Cargo.toml")]);
    assert_eq!(file_paths(&plan), vec![current_dir.join("Cargo.toml")]);
    assert!(plan.filter.is_relevant(&current_dir.join("Cargo.toml")));
}

/// Check the spelling of the strategy in configuration files
#[test]
fn test_strategy_in_job_config() {
    let job: crate::Job = toml::from_str(r#"file_watch_strategy = "file_poll""#).unwrap();
    assert_eq!(job.file_watch_strategy(), FileWatchStrategy::FilePoll);
    let job: crate::Job = toml::from_str(r#"file_watch_strategy = "parent_dir""#).unwrap();
    assert_eq!(job.file_watch_strategy(), FileWatchStrategy::ParentDir);
    let job: crate::Job = toml::from_str("").unwrap();
    assert_eq!(job.file_watch_strategy(), FileWatchStrategy::default());
    assert!(toml::from_str::<crate::Job>(r#"file_watch_strategy = "FilePoll""#).is_err());
}

/// Check that an event telling events were dropped is transmitted:
/// we can't know what changed, so we must assume the worst
#[test]
fn test_rescan_event_notifies() {
    let dir = TestDir::new();
    let mut filter = Filter::default();
    filter.add_file(&dir.path("Cargo.toml"));
    let mut ignorer = IgnorerSet::default();
    let kind = WatcherKind::Inotify;
    let rescan = Event::new(EventKind::Other).set_flag(Flag::Rescan);
    assert!(should_notify(&rescan, kind, &filter, &mut ignorer));
    // a rescan on macOS comes with the path of the watched directory,
    // which isn't a watched file
    let rescan = Event::new(EventKind::Other)
        .set_flag(Flag::Rescan)
        .add_path(dir.path.clone());
    assert!(should_notify(&rescan, kind, &filter, &mut ignorer));
}

/// Check the filtering done on the events which aren't rescans
#[test]
fn test_should_notify_filters_on_path() {
    let dir = TestDir::new();
    let mut filter = Filter::default();
    filter.add_file(&dir.path("Cargo.toml"));
    let mut ignorer = IgnorerSet::default();
    let kind = WatcherKind::Inotify;
    let event = |relative: &str| {
        Event::new(EventKind::Modify(ModifyKind::Any)).add_path(dir.path(relative))
    };
    assert!(should_notify(
        &event("Cargo.toml"),
        kind,
        &filter,
        &mut ignorer
    ));
    assert!(!should_notify(
        &event("README.md"),
        kind,
        &filter,
        &mut ignorer
    ));
    let pathless = Event::new(EventKind::Modify(ModifyKind::Any));
    assert!(!should_notify(&pathless, kind, &filter, &mut ignorer));
}

/// An unspecified data change is a write on kqueue, and noise elsewhere.
/// A write time change is a write when polling, and noise elsewhere.
#[test]
fn test_event_kinds_depend_on_backend() {
    let dir = TestDir::new();
    let mut filter = Filter::default();
    filter.add_file(&dir.path("Cargo.toml"));
    let mut ignorer = IgnorerSet::default();
    let data_any = Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Any)))
        .add_path(dir.path("Cargo.toml"));
    assert!(should_notify(
        &data_any,
        WatcherKind::Kqueue,
        &filter,
        &mut ignorer
    ));
    assert!(should_notify(
        &data_any,
        WatcherKind::PollWatcher,
        &filter,
        &mut ignorer
    ));
    assert!(!should_notify(
        &data_any,
        WatcherKind::Inotify,
        &filter,
        &mut ignorer
    ));
    assert!(!should_notify(
        &data_any,
        WatcherKind::Fsevent,
        &filter,
        &mut ignorer
    ));
    let write_time = Event::new(EventKind::Modify(ModifyKind::Metadata(
        MetadataKind::WriteTime,
    )))
    .add_path(dir.path("Cargo.toml"));
    assert!(should_notify(
        &write_time,
        WatcherKind::PollWatcher,
        &filter,
        &mut ignorer
    ));
    assert!(!should_notify(
        &write_time,
        WatcherKind::Kqueue,
        &filter,
        &mut ignorer
    ));
    assert!(!should_notify(
        &write_time,
        WatcherKind::Inotify,
        &filter,
        &mut ignorer
    ));
    // the path filter still applies when polling
    let other = Event::new(EventKind::Modify(ModifyKind::Metadata(
        MetadataKind::WriteTime,
    )))
    .add_path(dir.path("README.md"));
    assert!(!should_notify(
        &other,
        WatcherKind::PollWatcher,
        &filter,
        &mut ignorer
    ));
}
