use {
    super::FileWatchStrategy,
    rustc_hash::FxHashSet,
    std::path::{
        Path,
        PathBuf,
    },
};

/// Tells whether a path received in a notify event is one we watch.
///
/// Watched directories are watched recursively, so any path below them is
/// relevant. Watched files are listed, with their canonical form, so that
/// the other entries of a directory watched for one of its files can be
/// filtered out.
#[derive(Debug, Default)]
pub(super) struct Filter {
    dirs: Vec<PathBuf>,
    pub(super) files: FxHashSet<PathBuf>,
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
    pub(super) fn add_file(
        &mut self,
        path: &Path,
    ) {
        self.files.insert(path.to_path_buf());
        if let Some(canonic) = canonic(path) {
            self.files.insert(canonic);
        }
    }
    /// Tell whether the path is in a directory watched recursively
    pub(super) fn is_in_watched_dir(
        &self,
        path: &Path,
    ) -> bool {
        self.dirs.iter().any(|dir| path.starts_with(dir))
    }
    pub(super) fn is_watched_file(
        &self,
        path: &Path,
    ) -> bool {
        self.files.contains(path)
    }
    pub(super) fn is_relevant(
        &self,
        path: &Path,
    ) -> bool {
        self.is_watched_file(path) || self.is_in_watched_dir(path)
    }
}

/// Return the canonical form of the path when it differs from the path itself.
///
/// Events may come with canonicalized paths (that's the case on macOS) while
/// the paths to watch may go through symlinks, so both forms must be known.
/// The path may not exist yet, in which case its parent is canonicalized.
pub(super) fn canonic(path: &Path) -> Option<PathBuf> {
    let canonic = match path.canonicalize() {
        Ok(canonic) => canonic,
        Err(_) => path.parent()?.canonicalize().ok()?.join(path.file_name()?),
    };
    (canonic != path).then_some(canonic)
}

/// A directory watched non recursively, for some of the files it contains
#[derive(Debug)]
pub(super) struct WatchDir {
    pub(super) path: PathBuf,
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

/// A file watched directly
#[derive(Debug)]
pub(super) struct WatchFile {
    pub(super) path: PathBuf,
    canonic: PathBuf,
}

/// How to watch the paths of a mission: what to give to notify, and how to
/// filter the events it sends back
#[derive(Debug)]
pub(super) struct WatchPlan {
    strategy: FileWatchStrategy,
    /// directories, watched recursively
    pub(super) recursive: Vec<PathBuf>,
    /// directories watched for some of their files
    pub(super) non_recursive: Vec<WatchDir>,
    pub(super) files: Vec<WatchFile>,
    pub(super) filter: Filter,
}

impl WatchPlan {
    pub(super) fn new(strategy: FileWatchStrategy) -> Self {
        Self {
            strategy,
            recursive: Vec::new(),
            non_recursive: Vec::new(),
            files: Vec::new(),
            filter: Filter::default(),
        }
    }
    pub(super) fn add_path(
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
            // a recursive watch supersedes the watches below it,
            // whatever their spelling
            let canonic = canonic(path);
            let canonic = canonic.as_deref().unwrap_or(path);
            self.non_recursive
                .retain(|dir| !dir.path.starts_with(path) && !dir.canonic.starts_with(canonic));
            self.files
                .retain(|file| !file.path.starts_with(path) && !file.canonic.starts_with(canonic));
            debug!("add watch dir {path:?}");
            self.recursive.push(path.to_path_buf());
            return;
        }
        self.filter.add_file(path);
        let canonic = canonic(path);
        match self.strategy {
            FileWatchStrategy::ParentDir => {
                self.watch_parent_of(path);
                // a watched file may be a symlink, whose target is in another directory
                if let Some(canonic) = &canonic {
                    self.watch_parent_of(canonic);
                }
            }
            FileWatchStrategy::File | FileWatchStrategy::FilePoll => {
                self.watch_file(path, canonic.as_deref());
                // the changes of a symlink happen on its target
                let is_symlink = path.symlink_metadata().is_ok_and(|md| md.is_symlink());
                if let (true, Some(canonic)) = (is_symlink, &canonic) {
                    self.watch_file(canonic, None);
                }
            }
        }
    }
    /// Watch the directory containing the file instead of the file itself.
    /// The other entries of the directory are filtered out.
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
    fn watch_file(
        &mut self,
        file: &Path,
        canonic: Option<&Path>,
    ) {
        if self.filter.is_in_watched_dir(file)
            || canonic.is_some_and(|canonic| self.filter.is_in_watched_dir(canonic))
        {
            return; // already covered by a recursive watch
        }
        if !file.exists() {
            warn!("watch path doesn't exist: {file:?}");
            return;
        }
        if self.files.iter().any(|f| f.path == file) {
            return;
        }
        debug!("add watch file {file:?}");
        self.files.push(WatchFile {
            path: file.to_path_buf(),
            canonic: canonic.unwrap_or(file).to_path_buf(),
        });
    }
}
