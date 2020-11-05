use crate::*;

/// A report line
#[derive(Debug, Clone)]
pub struct Line {
    /// the index among items
    /// (all lines having the same index belong to
    /// the same error or warning item)
    pub item_idx: usize,

    /// the raw content, with its TTY styling
    pub content: String,

    pub line_type: LineType,
}
