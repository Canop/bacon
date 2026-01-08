/// A command to show/scroll to a specific diagnostic item by index
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShowItemCommand {
    pub item_idx: usize,
}

impl ShowItemCommand {
    /// Return the action description to show in doc/help
    pub fn doc(&self) -> String {
        format!("show item {}", self.item_idx)
    }
}
