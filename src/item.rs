#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Warning,
    Error,
}

/// A report line
#[derive(Debug, Clone)]
pub struct Line {
    /// whether it's displayed in summary mode
    pub summary: bool,

    /// the raw content, with its TTY styling
    pub content: String,
}

/// A warning or error
#[derive(Debug, Clone)]
pub struct Item {
    /// whether it's a warning or an error
    pub kind: Kind,

    /// all the lines of the report
    pub lines: Vec<Line>,
}
