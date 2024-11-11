use {
    crate::*,
    anyhow::Result,
    std::path::Path,
};

#[derive(Default)]
pub struct GlobIgnorer {
    globs: Vec<glob::Pattern>,
}

impl GlobIgnorer {
    pub fn add(
        &mut self,
        pattern: &str,
        root: &Path,
    ) -> Result<()> {
        if pattern.starts_with('/') {
            self.globs.push(glob::Pattern::new(pattern)?);
            // it's probably a path relative to the root of the package
            let pattern = root.join(pattern);
            let pattern = pattern.to_string_lossy();
            self.globs.push(glob::Pattern::new(&pattern)?);
        } else {
            // as glob doesn't work with non absolute paths, we make it absolute
            self.globs
                .push(glob::Pattern::new(&format!("/**/{}", pattern))?);
        }
        Ok(())
    }
}

impl Ignorer for GlobIgnorer {
    fn excludes(
        &mut self,
        paths: &Path,
    ) -> Result<bool> {
        for glob in &self.globs {
            if glob.matches_path(paths) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
