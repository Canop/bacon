use {
    crate::*,
    anyhow::*,
    crossterm::{
        style::{Colorize, Styler},
    },
    std::io::Write,
};

/// a kind of section
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    /// a warning
    Warning,
    /// an error
    Error,
    /// a test failure
    TestFail,
    /// a sum of errors and/or warnings, typically
    /// occuring at the end of the compilation of
    /// a package
    Sum,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineType {
    /// the start of a section
    Title(Kind),

    /// a line locating the problem
    Location,

    /// the line saying if a test was passed
    TestResult(bool),

    /// any other line
    Normal,
}

impl LineType {
    pub fn cols(self) -> usize {
        match self {
            Self::Title(_) => 3,
            _ => 0,
        }
    }
    pub fn draw(self, w: &mut W, item_idx:usize) -> Result<()> {
        match self {
            Self::Title(Kind::Error) => {
                write!(w, "{}", format!("{:^3}", item_idx).black().bold().on_red())?;
            }
            Self::Title(Kind::TestFail) => {
                write!(w, "\u{1b}[1m\u{1b}[38;5;235m\u{1b}[48;5;208m{:^3}\u{1b}[0m\u{1b}[0m", item_idx)?;
            }
            Self::Title(Kind::Warning) => {
                write!(w, "{}", format!("{:^3}", item_idx).black().bold().on_yellow())?;
            }
            _ => {}
        }
        Ok(())
    }
}

