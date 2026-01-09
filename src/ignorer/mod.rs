use {
    anyhow::Result,
    glob::Pattern,
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

/// Build glob patterns from a pattern string, handling both absolute
/// (starting with `/`) and relative patterns.
pub(crate) fn build_glob_patterns(
    pattern: &str,
    root: &Path,
) -> Result<Vec<Pattern>> {
    let mut patterns = Vec::new();
    if pattern.starts_with('/') {
        patterns.push(Pattern::new(pattern)?);
        let abs_pattern = root.join(pattern);
        patterns.push(Pattern::new(&abs_pattern.to_string_lossy())?);
    } else {
        patterns.push(Pattern::new(&format!("/**/{pattern}"))?);
    }
    Ok(patterns)
}

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
    /// Patterns that override ignore rules (from `!pattern` in the ignore config)
    override_globs: Vec<Pattern>,
}
impl IgnorerSet {
    pub fn add(
        &mut self,
        ignorer: Box<dyn Ignorer + Send>,
    ) {
        self.ignorers.push(ignorer);
    }
    /// Add an override pattern that will force-include matching paths,
    /// overriding any ignore rules (including .gitignore).
    /// This is used for negative patterns like `!myfile.txt` in the ignore config.
    pub fn add_override(
        &mut self,
        pattern: &str,
        root: &Path,
    ) -> Result<()> {
        self.override_globs
            .extend(build_glob_patterns(pattern, root)?);
        Ok(())
    }
    /// Check if a path matches any override pattern
    fn is_overridden(
        &self,
        path: &Path,
    ) -> bool {
        for glob in &self.override_globs {
            if glob.matches_path(path) {
                return true;
            }
        }
        false
    }
    pub fn excludes_all_pathbufs(
        &mut self,
        paths: &[PathBuf],
    ) -> Result<bool> {
        if self.ignorers.is_empty() {
            return Ok(false);
        }
        for path in paths {
            // First check if this path matches an override pattern.
            // Override patterns (from `!pattern` in ignore config) force-include
            // the path, regardless of any ignore rules.
            if self.is_overridden(path) {
                return Ok(false);
            }
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
