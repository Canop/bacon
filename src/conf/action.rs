use {
    crate::*,
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

/// An action that can be executed in the system, eg mapped to a key.
///
/// While it's not really specified, export names, just like
/// job names, must be gentle enough so as to be correctly parsed
/// (if not, they won't go very far in the system).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Back,       // leave help, clear search, go to previous job, leave, etc.
    BackOrQuit, // same as Back but quits if there is nothing to go back to
    CopyUnstyledOutput,
    DismissTop,
    DismissTopItem,
    DismissTopItemType,
    Export(String),
    FocusFile(FocusFileCommand),
    FocusGoto,
    FocusSearch,
    Help,
    Job(JobRef),
    NextMatch,
    NoOp, // no operation, can be used to clear a binding
    OpenJobsMenu,
    OpenMenu(Box<ActionMenuDefinition>),
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
    UndismissAll,
    UndismissLocation(String),
    UndismissDiagType(String),
    OpenUndismissMenu,
    Unpause,
    Validate, // validate search entry
}

impl Md for Action {
    /// Return the action description to show in doc/help
    fn md(&self) -> String {
        match self {
            Self::Back => "back to previous page or job".to_string(),
            Self::BackOrQuit => {
                "back to previous page or job, quitting if there is none".to_string()
            }
            Self::CopyUnstyledOutput => "copy current job's output".to_string(),
            Self::DismissTop => "dismiss top".to_string(),
            Self::DismissTopItem => "dismiss top item".to_string(),
            Self::DismissTopItemType => "dismiss top item type".to_string(),
            Self::Export(export_name) => format!("run *{export_name}* export"),
            Self::FocusFile(fc) => fc.doc(),
            Self::FocusGoto => "focus goto".to_string(),
            Self::FocusSearch => "focus search".to_string(),
            Self::Help => "help".to_string(),
            Self::Job(job_name) => format!("*{job_name}* job"),
            Self::NextMatch => "next match".to_string(),
            Self::NoOp => "no operation".to_string(),
            Self::OpenMenu(_) => "open specific menu".to_string(),
            Self::OpenJobsMenu => "open jobs menu".to_string(),
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
            Self::UndismissAll => "undismiss everything".to_string(),
            Self::UndismissLocation(loc) => format!("undismiss *{loc}*"),
            Self::UndismissDiagType(dt) => format!("undismiss *{dt}*"),
            Self::OpenUndismissMenu => "open undismiss menu".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseActionError {
    InvalidBacktraceLevel(String),
    InvalidPlaySoundParameter(String),
    InvalidScrollCommand(String),
    InvalidVolume(ParseVolumeError),
    UnknownAction(String),
    UnknownInternal(String),
}

impl fmt::Display for ParseActionError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::InvalidBacktraceLevel(level) => {
                write!(f, "Invalid backtrace level: {level:?}")
            }
            Self::InvalidPlaySoundParameter(param) => {
                write!(f, "Invalid play sound parameter: {param:?}")
            }
            Self::InvalidScrollCommand(cmd) => {
                write!(f, "Invalid scroll command: {cmd:?}")
            }
            Self::InvalidVolume(e) => {
                write!(f, "Invalid volume: {e}")
            }
            Self::UnknownAction(s) => {
                write!(
                    f,
                    "Action not understood: {s:?} (did you mean \"job:{s}\"?)"
                )
            }
            Self::UnknownInternal(s) => {
                write!(f, "Internal not understood: {s:?}")
            }
        }
    }
}
impl std::error::Error for ParseActionError {}

impl From<ParseVolumeError> for ParseActionError {
    fn from(e: ParseVolumeError) -> Self {
        Self::InvalidVolume(e)
    }
}

impl fmt::Display for Action {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::Back => write!(f, "back"),
            Self::BackOrQuit => write!(f, "back-or-quit"),
            Self::CopyUnstyledOutput => write!(f, "copy-unstyled-output"),
            Self::DismissTop => write!(f, "dismiss-top"),
            Self::DismissTopItem => write!(f, "dismiss-top-item"),
            Self::DismissTopItemType => write!(f, "dismiss-top-item-type"),
            Self::Export(name) => write!(f, "export:{name}"),
            Self::FocusFile(FocusFileCommand { file }) => {
                write!(f, "focus-file({file})")
            }
            Self::FocusGoto => write!(f, "focus-goto"),
            Self::FocusSearch => write!(f, "focus-search"),
            Self::Help => write!(f, "help"),
            Self::Job(job_ref) => write!(f, "job:{job_ref}"),
            Self::NextMatch => write!(f, "next-match"),
            Self::NoOp => write!(f, "no-op"),
            Self::OpenJobsMenu => write!(f, "open-jobs-menu"),
            Self::OpenMenu(def) => {
                write!(f, "open-menu(")?;
                if let Some(intro) = &def.intro {
                    write!(f, "intro={intro},")?;
                }
                write!(f, "actions=[")?;
                for (i, action) in def.actions.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{action}")?;
                }
                write!(f, "])")
            }
            Self::OpenUndismissMenu => write!(f, "open-undismiss-menu"),
            Self::Pause => write!(f, "pause"),
            Self::PlaySound(PlaySoundCommand { name, volume }) => {
                write!(f, "play-sound(")?;
                if let Some(name) = name {
                    write!(f, "name={name},")?;
                }
                write!(f, "volume={volume})")
            }
            Self::PreviousMatch => write!(f, "previous-match"),
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
            Self::UndismissAll => write!(f, "undismiss-all"),
            Self::UndismissLocation(loc) => write!(f, "undismiss-location({loc})"),
            Self::UndismissDiagType(dt) => write!(f, "undismiss-diag-type({dt})"),
            Self::Unpause => write!(f, "unpause"),
            Self::Validate => write!(f, "validate"),
        }
    }
}
impl FromStr for Action {
    type Err = ParseActionError;
    fn from_str(s: &str) -> Result<Self, ParseActionError> {
        regex_switch!(s,
            r"^export:(?<name>.+)$" => Self::Export(name.to_string()),
            r"^job:(?<job_ref>.+)$" => Self::Job(job_ref.into()),
            r"^(?:internal:)?back$" => Self::Back,
            r"^(?:internal:)?back-or-quit$" => Self::BackOrQuit,
            r"^(?:internal:)?dismiss-top$" => Self::DismissTop,
            r"^(?:internal:)?dismiss-top-item$" => Self::DismissTopItem,
            r"^(?:internal:)?dismiss-top-item-type$" => Self::DismissTopItemType,
            r"^(?:internal:)?help$" => Self::Help,
            r"^(?:internal:)?quit$" => Self::Quit,
            r"^(?:internal:)?open-menu\((?:intro=(?<intro>.+),\s*)?(?:actions=\[(?<actions>.+)\])\)$" => {
                let actions = actions.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::parse::<Action>)
                    .collect::<Result<Vec<_>, _>>()?;
                Self::OpenMenu(Box::new(ActionMenuDefinition {
                    intro: if intro.is_empty() { None } else { Some(intro.to_string()) },
                    actions,
                }))
            }
            r"^?open-jobs?-menu$" => Self::OpenJobsMenu,
            r"^(?:internal:)?refresh$" => Self::Refresh,
            r"^(?:internal:)?reload-config$" => Self::ReloadConfig,
            r"^(?:internal:)?rerun$" => Self::ReRun,
            r"^(?:internal:)?scope-to-failures$" => Self::ScopeToFailures,
            r"^(?:internal:)?toggle-raw-output$" => Self::ToggleRawOutput,
            r"^(?:internal:)?toggle-backtrace$" => Self::ToggleBacktrace("1"),
            r"^(?:internal:)?toggle-backtrace\(\s*(?<level>.+)\s*\)$" => {
                let level = match level {
                    "0" => "0",
                    "1" => "1",
                    "2" => "2",
                    "full" => "full",
                    _ => {
                        return Err(ParseActionError::InvalidBacktraceLevel(level.to_string()));
                    }
                };
                Self::ToggleBacktrace(level)
            }
            r"^(?:internal:)?toggle-summary$" => Self::ToggleSummary,
            r"^(?:internal:)?toggle-wrap$" => Self::ToggleWrap,
            r"^(?:internal:)?(noop|no-op|no-operation)$" => Self::NoOp,
            r"^(?:internal:)?pause$" => Self::Pause,
            r"^(?:internal:)?unpause$" => Self::Unpause,
            r"^(?:internal:)?toggle-pause$" => Self::TogglePause,
            r"^(?:internal:)?focus-search$" => Self::FocusSearch,
            r"^(?:internal:)?focus-goto$" => Self::FocusGoto,
            r"^(?:internal:)?validate$" => Self::Validate,
            r"^(?:internal:)?next-match$" => Self::NextMatch,
            r"^(?:internal:)?previous-match$" => Self::PreviousMatch,
            r"^(?:internal:)?undismiss-all$" => Self::UndismissAll,
            r"^(?:internal:)?undismiss-location\((?<location>.+)\)$" => Self::UndismissLocation(location.to_string()),
            r"^(?:internal:)?undismiss-diag-type\((?<diag_type>.+)\)$" => Self::UndismissDiagType(diag_type.to_string()),
            r"^(?:internal:)?open-undismiss-menu$" => Self::OpenUndismissMenu,
            r"^(?:internal:)?copy-unstyled-output$" => Self::CopyUnstyledOutput,
            r"^(?:internal:)?play-sound$" => Self::PlaySound(PlaySoundCommand::default()),
            r"^(?:internal:)?play-sound\((?<props>.*)\)$" => {
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
                            return Err(ParseActionError::InvalidPlaySoundParameter(
                                prop_name.to_string(),
                            ));
                        }
                    }
                }
                Self::PlaySound(PlaySoundCommand { name, volume })
            }
            r"^(?:internal:)?focus[_-]file\((?<file>.*)\)$" => Self::FocusFile(FocusFileCommand::new(file)),
            r"^(?:internal:)?(?<cmd>scroll.+)$" => {
                let cmd = ScrollCommand::from_str(cmd)
                    .map_err(|_| ParseActionError::InvalidScrollCommand(cmd.to_string()))?;
                Self::Scroll(cmd)
            }
        ).ok_or(ParseActionError::UnknownAction(s.to_string()))
    }
}

impl From<JobRef> for Action {
    fn from(jr: JobRef) -> Self {
        Self::Job(jr)
    }
}

impl Serialize for Action {
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
impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[test]
fn test_action_string_round_trip() {
    let actions = vec![
        Action::Job(JobRef::Default),
        Action::Job(JobRef::Initial),
        Action::Job(JobRef::PreviousOrQuit),
        Action::Job(JobRef::Previous),
        Action::Job(JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Name("run".to_string()),
            scope: Scope::default(),
        })),
        Action::Job(JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Name("nextest".to_string()),
            scope: Scope {
                tests: vec!["first::test".to_string(), "second_test".to_string()],
            },
        })),
        Action::Job(JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Alias("my-check".to_string()),
            scope: Scope::default(),
        })),
        Action::Job(JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Alias("my-test".to_string()),
            scope: Scope {
                tests: vec!["abc".to_string()],
            },
        })),
        Action::Job(JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Alias("a forbidden name!".to_string()),
            scope: Scope {
                tests: vec!["abc".to_string()],
            },
        })),
        Action::Help,
        Action::Scroll(ScrollCommand::MilliPages(1500)),
        Action::Scroll(ScrollCommand::MilliPages(-500)),
        Action::Scroll(ScrollCommand::MilliPages(-2000)),
        Action::Export("my export".to_string()),
        Action::Back,
        Action::BackOrQuit,
        Action::DismissTop,
        Action::DismissTopItem,
        Action::DismissTopItemType,
        Action::UndismissAll,
        Action::UndismissLocation("src/main.rs:42".to_string()),
        Action::FocusSearch,
        Action::OpenJobsMenu,
        Action::OpenMenu(Box::new(ActionMenuDefinition {
            intro: Some("This is a menu".to_string()),
            actions: vec![
                Action::Job(JobRef::Initial),
                Action::Job(JobRef::PreviousOrQuit),
                Action::Job(JobRef::Concrete(ConcreteJobRef {
                    name_or_alias: NameOrAlias::Name("run".to_string()),
                    scope: Scope::default(),
                })),
                Action::Help,
            ],
        })),
        Action::Help,
        Action::NoOp,
        Action::Pause,
        Action::Quit,
        Action::ReRun,
        Action::ReloadConfig,
        Action::ScopeToFailures,
        Action::Scroll(ScrollCommand::MilliPages(-3000)),
        Action::Scroll(ScrollCommand::MilliPages(-350)),
        Action::Scroll(ScrollCommand::MilliPages(1561)),
        Action::Scroll(ScrollCommand::Top),
        Action::ToggleBacktrace("1"),
        Action::ToggleBacktrace("full"),
        Action::TogglePause,
        Action::ToggleSummary,
        Action::ToggleWrap,
        Action::Unpause,
        Action::Validate,
        Action::NextMatch,
        Action::PreviousMatch,
        Action::PlaySound(PlaySoundCommand::default()),
        Action::PlaySound(PlaySoundCommand {
            name: None,
            volume: Volume::new(50),
        }),
        Action::PlaySound(PlaySoundCommand {
            name: Some("beep-beep".to_string()),
            volume: Volume::new(100),
        }),
        Action::PlaySound(PlaySoundCommand {
            name: None,
            volume: Volume::new(0),
        }),
    ];
    for action in actions {
        println!("action: {}", action.to_string());
        assert_eq!(action.to_string().parse(), Ok(action));
    }
}

/// Check that white space is allowed around play-sound parameters
/// See https://github.com/Canop/bacon/issues/322
#[test]
fn test_play_sound_parsing_with_space() {
    use {
        crate::Action,
        pretty_assertions::assert_eq,
    };
    let strings = [
        "play-sound(name=car-horn,volume=5)",
        "play-sound(name=car-horn, volume=5)",
        "internal:play-sound(name=car-horn, volume=5)",
        "play-sound( name = car-horn , volume = 5 )",
    ];
    let psc = PlaySoundCommand {
        name: Some("car-horn".to_string()),
        volume: Volume::new(5),
    };
    for string in &strings {
        let action: Action = string.parse().unwrap();
        assert_eq!(action, Action::PlaySound(psc.clone()));
    }
}
