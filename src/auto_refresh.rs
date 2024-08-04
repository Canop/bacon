#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoRefresh {
    Paused,
    PausedWithMisses,
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
