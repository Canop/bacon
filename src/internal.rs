use {
    crate::ScrollCommand,
    std::fmt,
};

/// one of the hardcoded actions that can be mapped
/// to a key or ran after a successful job
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Internal {
    Back,
    Help,
    Pause,
    Quit,
    ReRun,
    Refresh, // clear and rerun
    ScopeToFailures,
    Scroll(ScrollCommand),
    ToggleBacktrace(&'static str),
    TogglePause, // either pause or unpause
    ToggleRawOutput,
    ToggleSummary,
    ToggleWrap,
    Unpause,
}

/// A string representation for the help page
impl fmt::Display for Internal {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::Back => write!(f, "back to previous page or job"),
            Self::Help => write!(f, "help"),
            Self::Pause => write!(f, "pause"),
            Self::Quit => write!(f, "quit"),
            Self::ReRun => write!(f, "run current job again"),
            Self::Refresh => write!(f, "clear then run current job again"),
            Self::ScopeToFailures => write!(f, "scope to failures"),
            Self::Scroll(scroll_command) => scroll_command.fmt(f),
            Self::ToggleBacktrace(level) => write!(f, "toggle backtrace ({level})"),
            Self::TogglePause => write!(f, "toggle pause"),
            Self::ToggleRawOutput => write!(f, "toggle raw output"),
            Self::ToggleSummary => write!(f, "toggle summary"),
            Self::ToggleWrap => write!(f, "toggle wrap"),
            Self::Unpause => write!(f, "unpause"),
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
            "back" => Ok(Self::Back),
            "help" => Ok(Self::Help),
            "quit" => Ok(Self::Quit),
            "refresh" => Ok(Self::Refresh),
            "rerun" => Ok(Self::ReRun),
            "scope-to-failures" => Ok(Self::ScopeToFailures),
            "toggle-raw-output" => Ok(Self::ToggleRawOutput),
            "toggle-backtrace" => Ok(Self::ToggleBacktrace("1")),
            "toggle-backtrace(1)" => Ok(Self::ToggleBacktrace("1")),
            "toggle-backtrace(2)" => Ok(Self::ToggleBacktrace("2")),
            "toggle-backtrace(full)" => Ok(Self::ToggleBacktrace("full")),
            "toggle-summary" => Ok(Self::ToggleSummary),
            "toggle-wrap" => Ok(Self::ToggleWrap),
            "pause" => Ok(Self::Pause),
            "unpause" => Ok(Self::Unpause),
            "toggle pause" => Ok(Self::TogglePause),
            _ => Err(()),
        }
    }
}
