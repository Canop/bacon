#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoRefresh {
    /// Don't rerun the job on file changes.
    Paused,
    /// Don't rerun the job, also we already missed some changes
    /// (so if we enable, we should immediately rerun the job).
    PausedWithMisses,
    /// Run the job on file changes.
    Enabled,
}

impl AutoRefresh {
    pub fn is_enabled(self) -> bool {
        matches!(self, AutoRefresh::Enabled)
    }

    pub fn is_paused(self) -> bool {
        matches!(self, AutoRefresh::Paused | AutoRefresh::PausedWithMisses)
    }
}
