use {
    crate::ScrollCommand,
    serde::{
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
        de,
    },
    std::{
        fmt,
        str::FromStr,
    },
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

impl Internal {
    /// Return the action description to show in doc/help
    pub fn doc(&self) -> String {
        match self {
            Self::Back => "back to previous page or job".to_string(),
            Self::Help => "help".to_string(),
            Self::Pause => "pause".to_string(),
            Self::Quit => "quit".to_string(),
            Self::ReRun => "run current job again".to_string(),
            Self::Refresh => "clear then run current job again".to_string(),
            Self::ScopeToFailures => "scope to failures".to_string(),
            Self::Scroll(scroll_command) => scroll_command.doc(),
            Self::ToggleBacktrace(level) => format!("toggle backtrace ({level})"),
            Self::TogglePause => "toggle pause".to_string(),
            Self::ToggleRawOutput => "toggle raw output".to_string(),
            Self::ToggleSummary => "toggle summary".to_string(),
            Self::ToggleWrap => "toggle wrap".to_string(),
            Self::Unpause => "unpause".to_string(),
        }
    }
}

impl fmt::Display for Internal {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::Back => write!(f, "back"),
            Self::Help => write!(f, "help"),
            Self::Pause => write!(f, "pause"),
            Self::Quit => write!(f, "quit"),
            Self::ReRun => write!(f, "rerun"),
            Self::Refresh => write!(f, "refresh"),
            Self::ScopeToFailures => write!(f, "scope-to-failures"),
            Self::Scroll(scroll_command) => scroll_command.fmt(f),
            Self::ToggleBacktrace(level) => write!(f, "toggle-backtrace({level})"),
            Self::TogglePause => write!(f, "toggle-pause"),
            Self::ToggleRawOutput => write!(f, "toggle-raw-output"),
            Self::ToggleSummary => write!(f, "toggle-summary"),
            Self::ToggleWrap => write!(f, "toggle-wrap"),
            Self::Unpause => write!(f, "unpause"),
        }
    }
}
impl std::str::FromStr for Internal {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            "toggle-pause" => Ok(Self::TogglePause),
            _ => Err("invalid internal"),
        }
    }
}
impl Serialize for Internal {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for Internal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}

#[test]
fn test_internal_string_round_trip() {
    let internals = [
        Internal::Back,
        Internal::Help,
        Internal::Pause,
        Internal::Quit,
        Internal::ReRun,
        Internal::ScopeToFailures,
        Internal::Scroll(ScrollCommand::Pages(-3)),
        Internal::Scroll(ScrollCommand::Top),
        Internal::ToggleBacktrace("1"),
        Internal::ToggleBacktrace("full"),
        Internal::TogglePause,
        Internal::ToggleSummary,
        Internal::ToggleWrap,
        Internal::Unpause,
    ];
    for internal in internals {
        assert_eq!(internal.to_string().parse(), Ok(internal));
    }
}
