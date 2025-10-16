use {
    crate::*,
    anyhow::{
        Context,
        Result,
    },
    gix::{
        self as git,
        Repository,
    },
    std::path::Path,
};

/// An object able to tell whether a file is excluded
/// by gitignore rules
pub struct GitIgnorer {
    repo: Repository,
}

impl GitIgnorer {
    /// Create an Ignorer from any directory path: the closest
    /// surrounding git repository will be found (if there's one)
    /// and its gitignore rules used.
    ///
    /// `root_path` is assumed to exist and be a directory
    pub(crate) fn new(root_path: &Path) -> Result<Self> {
        let repo = git::discover(root_path)?;
        Ok(Self { repo })
    }
}

impl Ignorer for GitIgnorer {
    fn excludes(
        &mut self,
        paths: &Path,
    ) -> Result<bool> {
        self.excludes_all_paths(&[paths])
    }
}
impl GitIgnorer {
    fn excludes_all_paths(
        &mut self,
        paths: &[&Path],
    ) -> Result<bool> {
        let worktree = self.repo.worktree().context("a worktree should exist")?;

        // The "Cache" is the structure allowing checking exclusion.
        // Building it is the most expensive operation, and we could store it
        // in the Ignorer instead of the repo (by having the repo in the mission),
        // but it's still about just 1ms and I'm not sure we know if it always
        // stays valid.
        let mut cache = time!(Debug, worktree.excludes(None)?);

        for path in paths {
            // cache.at_path panics if not provided a path relative
            // to the work directory, so we compute the relative path
            let Some(work_dir) = self.repo.workdir() else {
                return Ok(false);
            };
            let Ok(relative_path) = path.strip_prefix(work_dir) else {
                return Ok(false);
            };

            // cache.at_path panics if the relative path is empty, so
            // we must check that
            if relative_path.as_os_str().is_empty() {
                return Ok(true);
            }

            if path.is_dir() {
                // we're not interested in directories (we should not receive them anyway)
                return Ok(false);
            }

            let platform = cache.at_path(relative_path, Some(gix::index::entry::Mode::FILE))?;

            if !platform.is_excluded() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
