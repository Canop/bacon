use {
    anyhow::Result,
    std::path::{
        Path,
        PathBuf,
    },
};

mod git_ignorer;
mod glob_ignorer;

pub use {
    git_ignorer::GitIgnorer,
    glob_ignorer::GlobIgnorer,
};

pub trait Ignorer {
    /// Tell whether all given paths are excluded according to
    /// either the global gitignore rules or the ones of the repository.
    ///
    /// Return Ok(false) when at least one file is included (i.e. we should
    /// execute the job)
    fn excludes(
        &mut self,
        paths: &Path,
    ) -> Result<bool>;
}

/// A set of ignorers
#[derive(Default)]
pub struct IgnorerSet {
    ignorers: Vec<Box<dyn Ignorer + Send>>,
}
impl IgnorerSet {
    pub fn add(
        &mut self,
        ignorer: Box<dyn Ignorer + Send>,
    ) {
        self.ignorers.push(ignorer);
    }
    pub fn excludes_all_pathbufs(
        &mut self,
        paths: &[PathBuf],
    ) -> Result<bool> {
        if self.ignorers.is_empty() {
            return Ok(false);
        }
        for path in paths {
            let mut excluded = false;
            for ignorer in &mut self.ignorers {
                if ignorer.excludes(path)? {
                    excluded = true;
                    break;
                }
            }
            if !excluded {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
