#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoRefresh {
    /// Don't rerun the job on file changes.
    Paused,
    /// Run the job on file changes.
    Enabled,
}

impl AutoRefresh {
    pub fn is_enabled(self) -> bool {
        matches!(self, AutoRefresh::Enabled)
    }

    pub fn is_paused(self) -> bool {
        matches!(self, AutoRefresh::Paused)
    }
}
