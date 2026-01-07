/// A command to select/scroll to a specific diagnostic item by index
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelectItemCommand {
    pub item_idx: usize,
}

impl SelectItemCommand {
    /// Return the action description to show in doc/help
    pub fn doc(&self) -> String {
        format!("select item {}", self.item_idx)
    }
}
