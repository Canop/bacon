use crate::*;

#[derive(Default)]
pub struct AppState {
    pub headless: bool,
    /// Dimissals and filtering state
    pub filter: Filter,
}
