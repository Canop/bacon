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

/// Build glob patterns from a pattern string, generating a little more patterns
/// than might be expected, to cover intended use cases.
///
/// For example, for a pattern like `foo/bar`, we want to
/// match both `foo/bar` and `foo/bar/**` (i.e. the directory and all its contents).
/// Patterns starting with a slash are either relative to the root, or really absolute
/// and relative to the filesystems root.
///
/// See `test_expand_patterns` for examples of the generated patterns.
pub(crate) fn build_glob_patterns(
    pattern: &str,
    root: &Path,
) -> Result<Vec<Pattern>> {
    let patterns = expand_patterns(pattern, root);
    let mut glob_patterns = Vec::with_capacity(patterns.len());
    for p in patterns {
        glob_patterns.push(Pattern::new(&p)?);
    }
    Ok(glob_patterns)
}

fn expand_patterns(
    pattern: &str,
    root: &Path,
) -> Vec<String> {
    let mut patterns_over_start = Vec::new();
    let starts_with_slash = pattern.starts_with('/');
    let ends_in_slash = pattern.ends_with('/');
    if starts_with_slash {
        let without_slash = pattern.trim_start_matches('/');
        patterns_over_start.push(root.join(without_slash).to_string_lossy().to_string());
        patterns_over_start.push(pattern.to_string());
    } else if pattern.starts_with("**/") {
        patterns_over_start.push(pattern.to_string());
    } else {
        patterns_over_start.push(format!("**/{pattern}"));
    }
    let mut patterns = Vec::new();
    for p in patterns_over_start {
        if !pattern.ends_with('*') {
            let complement = if ends_in_slash { "**" } else { "/**" };
            patterns.push(format!("{p}{complement}"));
        }
        if !ends_in_slash {
            patterns.push(p);
        }
    }
    patterns
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

#[test]
fn test_expand_patterns() {
    assert_eq!(
        expand_patterns("foo/bar", Path::new("/root")),
        vec!["**/foo/bar/**".to_string(), "**/foo/bar".to_string(),]
    );
    assert_eq!(
        expand_patterns("foo/bar/", Path::new("/root")),
        vec!["**/foo/bar/**".to_string(),]
    );
    assert_eq!(
        expand_patterns("/foo", Path::new("/root")),
        vec![
            "/root/foo/**".to_string(),
            "/root/foo".to_string(),
            "/foo/**".to_string(),
            "/foo".to_string(),
        ]
    );
    assert_eq!(
        expand_patterns("**/toto", Path::new("/root")),
        vec!["**/toto/**".to_string(), "**/toto".to_string(),]
    );
    assert_eq!(
        expand_patterns("toto/**", Path::new("/root")),
        vec!["**/toto/**".to_string(),]
    );
    assert_eq!(
        expand_patterns("/foo/bar/*", Path::new("/root")),
        vec!["/root/foo/bar/*".to_string(), "/foo/bar/*".to_string(),]
    );
    assert_eq!(
        expand_patterns("foo/**/bar/*", Path::new("/root")),
        vec!["**/foo/**/bar/*".to_string(),]
    );
}
