use {
    crate::ScrollCommand,
    std::fmt,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Internal {
    Quit,
    Scroll(ScrollCommand),
    ToggleBacktrace,
    ToggleSummary,
    ToggleWrap,
}

impl fmt::Display for Internal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Quit => write!(f, "quit"),
            Self::Scroll(scroll_command) => scroll_command.fmt(f),
            Self::ToggleBacktrace => write!(f, "toggle backtrace"),
            Self::ToggleSummary => write!(f, "toggle summary"),
            Self::ToggleWrap => write!(f, "toggle wrap"),
        }
    }
}

impl std::str::FromStr for Internal {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        if let Ok(scroll_command) = ScrollCommand::from_str(s) {
            return Ok(Self::Scroll(scroll_command));
        }
        match s {
            "quit" => Ok(Self::Quit),
            "toggle-backtrace" => Ok(Self::ToggleBacktrace),
            "toggle-summary" => Ok(Self::ToggleSummary),
            "toggle-wrap" => Ok(Self::ToggleWrap),
            _ => Err(()),
        }
    }
}
