use {
    super::build_glob_patterns,
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
        self.globs.extend(build_glob_patterns(pattern, root)?);
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
