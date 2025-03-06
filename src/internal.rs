use {
    crate::{
        PlaySoundCommand,
        ScrollCommand,
        Volume,
    },
    lazy_regex::*,
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
    Back,       // leave help, clear search, go to previous job, leave, etc.
    BackOrQuit, // same as Back but quits if there is nothing to go back to
    CopyUnstyledOutput,
    FocusSearch,
    FocusGoto,
    Help,
    NextMatch,
    NoOp, // no operation, can be used to clear a binding
    Pause,
    PlaySound(PlaySoundCommand),
    PreviousMatch,
    Quit,
    ReRun,
    Refresh, // clear and rerun
    ReloadConfig,
    ScopeToFailures,
    Scroll(ScrollCommand),
    ToggleBacktrace(&'static str),
    TogglePause, // either pause or unpause
    ToggleRawOutput,
    ToggleSummary,
    ToggleWrap,
    Unpause,
    Validate, // validate search entry
}

impl Internal {
    /// Return the action description to show in doc/help
    pub fn doc(&self) -> String {
        match self {
            Self::Back => "back to previous page or job".to_string(),
            Self::BackOrQuit => {
                "back to previous page or job, quitting if there is none".to_string()
            }
            Self::CopyUnstyledOutput => "copy current job's output".to_string(),
            Self::FocusSearch => "focus search".to_string(),
            Self::FocusGoto => "focus goto".to_string(),
            Self::Help => "help".to_string(),
            Self::NextMatch => "next match".to_string(),
            Self::NoOp => "no operation".to_string(),
            Self::Pause => "pause".to_string(),
            Self::PlaySound(_) => "play sound".to_string(),
            Self::PreviousMatch => "previous match".to_string(),
            Self::Quit => "quit".to_string(),
            Self::ReRun => "run current job again".to_string(),
            Self::Refresh => "clear then run current job again".to_string(),
            Self::ReloadConfig => "reload configuration files".to_string(),
            Self::ScopeToFailures => "scope to failures".to_string(),
            Self::Scroll(scroll_command) => scroll_command.doc(),
            Self::ToggleBacktrace(level) => format!("toggle backtrace ({level})"),
            Self::TogglePause => "toggle pause".to_string(),
            Self::ToggleRawOutput => "toggle raw output".to_string(),
            Self::ToggleSummary => "toggle summary".to_string(),
            Self::ToggleWrap => "toggle wrap".to_string(),
            Self::Unpause => "unpause".to_string(),
            Self::Validate => "validate".to_string(),
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
            Self::BackOrQuit => write!(f, "back-or-quit"),
            Self::CopyUnstyledOutput => write!(f, "copy-unstyled-output"),
            Self::Help => write!(f, "help"),
            Self::NoOp => write!(f, "no-op"),
            Self::Pause => write!(f, "pause"),
            Self::Quit => write!(f, "quit"),
            Self::ReRun => write!(f, "rerun"),
            Self::Refresh => write!(f, "refresh"),
            Self::ReloadConfig => write!(f, "reload-config"),
            Self::ScopeToFailures => write!(f, "scope-to-failures"),
            Self::Scroll(scroll_command) => scroll_command.fmt(f),
            Self::ToggleBacktrace(level) => write!(f, "toggle-backtrace({level})"),
            Self::TogglePause => write!(f, "toggle-pause"),
            Self::ToggleRawOutput => write!(f, "toggle-raw-output"),
            Self::ToggleSummary => write!(f, "toggle-summary"),
            Self::ToggleWrap => write!(f, "toggle-wrap"),
            Self::Unpause => write!(f, "unpause"),
            Self::FocusSearch => write!(f, "focus-search"),
            Self::FocusGoto => write!(f, "focus-goto"),
            Self::Validate => write!(f, "validate"),
            Self::NextMatch => write!(f, "next-match"),
            Self::PreviousMatch => write!(f, "previous-match"),
            Self::PlaySound(PlaySoundCommand { name, volume }) => {
                write!(f, "play-sound(")?;
                if let Some(name) = name {
                    write!(f, "name={},", name)?;
                }
                write!(f, "volume={})", volume)
            }
        }
    }
}
impl std::str::FromStr for Internal {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(scroll_command) = ScrollCommand::from_str(s) {
            return Ok(Self::Scroll(scroll_command));
        }
        match s {
            "back" => Ok(Self::Back),
            "back-or-quit" => Ok(Self::BackOrQuit),
            "help" => Ok(Self::Help),
            "quit" => Ok(Self::Quit),
            "refresh" => Ok(Self::Refresh),
            "reload-config" => Ok(Self::ReloadConfig),
            "rerun" => Ok(Self::ReRun),
            "scope-to-failures" => Ok(Self::ScopeToFailures),
            "toggle-raw-output" => Ok(Self::ToggleRawOutput),
            "toggle-backtrace" => Ok(Self::ToggleBacktrace("1")),
            "toggle-backtrace(1)" => Ok(Self::ToggleBacktrace("1")),
            "toggle-backtrace(2)" => Ok(Self::ToggleBacktrace("2")),
            "toggle-backtrace(full)" => Ok(Self::ToggleBacktrace("full")),
            "toggle-summary" => Ok(Self::ToggleSummary),
            "toggle-wrap" => Ok(Self::ToggleWrap),
            "noop" | "no-op" | "no-operation" => Ok(Self::NoOp),
            "pause" => Ok(Self::Pause),
            "unpause" => Ok(Self::Unpause),
            "toggle-pause" => Ok(Self::TogglePause),
            "focus-search" => Ok(Self::FocusSearch),
            "focus-goto" => Ok(Self::FocusGoto),
            "validate" => Ok(Self::Validate),
            "next-match" => Ok(Self::NextMatch),
            "previous-match" => Ok(Self::PreviousMatch),
            "copy-unstyled-output" => Ok(Self::CopyUnstyledOutput),
            "play-sound" => Ok(Self::PlaySound(PlaySoundCommand::default())),
            _ => {
                if let Some((_, props)) = regex_captures!(r"^play[_-]sound\((.*)\)$", s) {
                    let iter = regex_captures_iter!(r"([^=,]+)=([^=,]+)", props);
                    let mut volume = Volume::default();
                    let mut name = None;
                    for (_, [prop_name, prop_value]) in iter.map(|c| c.extract()) {
                        let prop_value = prop_value.trim();
                        match prop_name.trim() {
                            "name" => {
                                name = Some(prop_value.to_string());
                            }
                            "volume" => {
                                volume = prop_value.parse()?;
                            }
                            _ => {
                                return Err("invalid play-sound parameter: {prop_name}".to_string());
                            }
                        }
                    }
                    return Ok(Self::PlaySound(PlaySoundCommand { name, volume }));
                }
                Err("invalid internal".to_string())
            }
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
        serializer.collect_str(self)
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
    use crate::Volume;
    let internals = [
        Internal::Back,
        Internal::BackOrQuit,
        Internal::FocusSearch,
        Internal::Help,
        Internal::NoOp,
        Internal::Pause,
        Internal::Quit,
        Internal::ReRun,
        Internal::ReloadConfig,
        Internal::ScopeToFailures,
        Internal::Scroll(ScrollCommand::MilliPages(-3000)),
        Internal::Scroll(ScrollCommand::MilliPages(-350)),
        Internal::Scroll(ScrollCommand::MilliPages(1561)),
        Internal::Scroll(ScrollCommand::Top),
        Internal::ToggleBacktrace("1"),
        Internal::ToggleBacktrace("full"),
        Internal::TogglePause,
        Internal::ToggleSummary,
        Internal::ToggleWrap,
        Internal::Unpause,
        Internal::Validate,
        Internal::NextMatch,
        Internal::PreviousMatch,
        Internal::PlaySound(PlaySoundCommand::default()),
        Internal::PlaySound(PlaySoundCommand {
            name: None,
            volume: Volume::new(50),
        }),
        Internal::PlaySound(PlaySoundCommand {
            name: Some("beep-beep".to_string()),
            volume: Volume::new(100),
        }),
        Internal::PlaySound(PlaySoundCommand {
            name: None,
            volume: Volume::new(0),
        }),
    ];
    for internal in internals {
        println!("testing {:?}", internal.to_string());
        assert_eq!(internal.to_string().parse(), Ok(internal));
    }
}

/// Check that white space is allowed around play-sound parameters
/// See https://github.com/Canop/bacon/issues/322
#[test]
fn test_play_sound_parsing_with_space() {
    use crate::Action;
    let strings = [
        "play-sound(name=car-horn,volume=5)",
        "play-sound(name=car-horn, volume=5)",
        "play-sound( name = car-horn , volume = 5 )",
    ];
    let psc = PlaySoundCommand {
        name: Some("car-horn".to_string()),
        volume: Volume::new(5),
    };
    for string in &strings {
        let action: Action = string.parse().unwrap();
        assert_eq!(action, Action::Internal(Internal::PlaySound(psc.clone())));
    }
}
