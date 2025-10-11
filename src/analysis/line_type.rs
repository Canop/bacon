use {
    crate::*,
    anyhow::*,
    serde::{
        Deserialize,
        Serialize,
    },
    std::io::Write,
    termimad::crossterm::style::Stylize,
};

/// a kind of section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// a warning
    Warning,
    /// an error
    Error,
    /// a test failure
    TestFail,
    /// a test output (may be a failure, or just --show-output)
    TestOutput,
    /// a sum of errors and/or warnings, typically occurring
    /// at the end of the compilation of a package
    Sum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineType {
    /// the start of a section
    Title(Kind),

    /// the end of a section (not part of the section)
    SectionEnd,

    /// a line locating the problem
    Location,

    /// the line saying if a test was passed
    TestResult(bool),

    /// a suggestion to try with backtrace
    BacktraceSuggestion,

    /// a line we know is useless noise
    Garbage,

    /// Raw line, unclassified
    Raw(CommandStream),

    /// Continuation of a previous line
    Continuation {
        /// offset to count back to get to first (starting at 1)
        offset: usize,
        /// whether the line is a summary
        summary: bool,
    },

    /// any other line
    Normal,
}

impl LineType {
    /// Width on screen for the specific prefix of line of this type
    #[must_use]
    pub fn cols(self) -> usize {
        match self {
            Self::Title(_) => 3,
            _ => 0,
        }
    }
    #[must_use]
    pub fn at_index_in(
        idx: usize,
        lines: &[Line],
    ) -> Option<Self> {
        let line = lines.get(idx)?;
        match line.line_type {
            Self::Continuation { offset, .. } => {
                if offset > idx {
                    error!("inconsistent offset in continuation line");
                    return None;
                }
                let idx = idx - offset;
                let line = lines.get(idx)?;
                Some(line.line_type)
            }
            line_type => Some(line_type),
        }
    }
    #[must_use]
    pub fn is_summary(self) -> bool {
        match self {
            Self::Normal | Self::Raw(_) => false,
            Self::Continuation { summary, .. } => summary,
            _ => true,
        }
    }
    #[must_use]
    pub fn matches(
        self,
        summary: bool,
    ) -> bool {
        !summary || self.is_summary()
    }
    pub fn draw(
        self,
        w: &mut W,
        item_idx: usize,
    ) -> Result<()> {
        match self {
            Self::Title(Kind::Error) => {
                write!(w, "{}", format!("{item_idx:^3}").black().bold().on_red())?;
            }
            Self::Title(Kind::TestFail | Kind::TestOutput) => {
                write!(
                    w,
                    "\u{1b}[1m\u{1b}[38;5;235m\u{1b}[48;5;208m{item_idx:^3}\u{1b}[0m\u{1b}[0m"
                )?;
            }
            Self::Title(Kind::Warning) => {
                write!(w, "{}", format!("{item_idx:^3}").black().bold().on_yellow())?;
            }
            _ => {}
        }
        Ok(())
    }
}
