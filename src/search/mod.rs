mod goto_idx;
mod line_pattern;
mod search_pattern;

pub use {
    goto_idx::*,
    line_pattern::*,
    search_pattern::*,
};

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchMode {
    Pattern,
    ItemIdx,
}

/// position in a `[TLine]` of a found pattern
#[derive(Debug, PartialEq, Eq)]
pub struct Found {
    /// The index of the first line containing the pattern
    pub line_idx: usize,
    /// The range of the pattern in the line
    pub trange: TRange,
    /// If the pattern goes over a line wrap, the range of the pattern in the next line
    pub continued: Option<TRange>,
}

pub enum Search {
    Pattern(Pattern),
    ItemIdx(usize),
}

impl Search {
    pub fn search_lines<'i, I>(
        &self,
        lines: I,
    ) -> Vec<Found>
    where
        I: IntoIterator<Item = &'i Line>,
    {
        match self {
            Self::Pattern(pattern) => pattern.search_lines(lines),
            Self::ItemIdx(idx) => search_item_idx(*idx, lines),
        }
    }
}
