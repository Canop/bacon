use crate::*;

/// the dialog that may be displayed over the rest of the UI
#[allow(clippy::large_enum_variant)]
pub enum Dialog {
    None,
    Menu(ActionMenu),
    //UndismissMenu(UndismissMenu),
}

impl Dialog {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}
