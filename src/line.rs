use crate::*;

/// A report line
#[derive(Debug)]
pub struct Line {
    /// the index among items
    /// (all lines having the same index belong to
    /// the same error or warning item)
    pub item_idx: usize,

    pub line_type: LineType,

    pub content: TLine,
}
