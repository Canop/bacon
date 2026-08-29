use crate::*;

#[derive(Default)]
pub struct AppState {
    pub headless: bool,
    /// Dismissals and filtering state
    pub filter: Filter,
}
