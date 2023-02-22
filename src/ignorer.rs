use {
    anyhow::{
        Context,
        Result,
    },
    gix::{
        self as git,
        prelude::FindExt,
        Repository,
    },
    std::path::{
        Path,
        PathBuf,
    },
};

/// An object able to tell whether a file is excluded
/// by gitignore rules
pub struct Ignorer {
    repo: Repository,
}

impl Ignorer {
    /// Create an Ignorer from any directory path: the closest
    /// surrounding git repository will be found (if there's one)
    /// and its gitignore rules used.
    ///
    /// root_path is assumed to exist and be a directory
    pub(crate) fn new(root_path: &Path) -> Result<Self> {
        let repo = git::discover(root_path)?;
        Ok(Self { repo })
    }

    /// Tell whether the given path is excluded according to
    /// either the global gitignore rules or the ones of the repository
    pub fn excludes(
        &mut self,
        file_path: &Path,
    ) -> Result<bool> {
        let worktree = self.repo.worktree().context("a worktree should exist")?;
        let index = worktree.index()?;

        // the "Cache" is the structure allowing checking exclusion
        let mut cache = worktree.excludes(&index, None)?;

        // cache.at_path panics if not provided a path relative
        // to the work directory, so we compute the relative path
        let Some(work_dir) = self.repo.work_dir() else {
            return Ok(false);
        };
        let Ok(relative_path) = file_path.strip_prefix(work_dir) else {
            return Ok(false);
        };

        // cache.at_path panics if the relative path is empty, so
        // we must check that
        if relative_path.as_os_str().is_empty() {
            return Ok(true);
        };

        let platform = cache.at_path(relative_path, Some(file_path.is_dir()), |oid, buf| {
            self.repo.objects.find_blob(oid, buf)
        })?;

        Ok(platform.is_excluded())
    }

    /// Return Ok(false) when at least one file is included (i.e. we should
    /// execute the job)
    pub fn excludes_all(
        &mut self,
        paths: &[PathBuf],
    ) -> Result<bool> {
        for path in paths {
            if !self.excludes(path)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
