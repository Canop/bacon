use crate::*;

/// A report line
#[derive(Debug)]
pub struct Line {
    /// the index among items
    /// (all lines having the same index belong to
    /// the same error, warning, or test item)
    pub item_idx: usize,

    pub line_type: LineType,

    pub content: TLine,
}

impl Line {
    pub fn location(&self) -> Option<&str> {
        match self.line_type {
            LineType::Location => {
                info!("CON: {:#?}", &self.content);
                self.content.strings.get(2).map(|ts| ts.raw.as_str())
            }
            _ => None,
        }
    }
}

impl WrappableLine for Line {
    fn content(&self) -> &TLine {
        &self.content
    }
    fn prefix_cols(&self) -> usize {
        self.line_type.cols()
    }
}
