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

/// An action that can be mapped to a key
///
/// While it's not really specified, export names, just like
/// job names, must be gentle enough so as to be correctly parsed
/// (if not, they won't go very far in the system).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Export(String),
    Internal(Internal),
    Job(JobRef),
}

impl Action {
    /// Return the action description to show in doc/help
    pub fn doc(&self) -> String {
        match self {
            Self::Export(export_name) => format!("run *{export_name}* export"),
            Self::Internal(internal) => internal.doc(),
            Self::Job(job_name) => format!("start the *{job_name}* job"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseActionError {
    UnknownAction(String),
    UnknowCategory(String),
    UnknownInternal(String),
}

impl fmt::Display for ParseActionError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::UnknownAction(s) => {
                write!(
                    f,
                    "Action not understood: {s:?} (did you mean \"job:{s}\"?)"
                )
            }
            Self::UnknowCategory(s) => {
                write!(f, "Unknown category: {s:?}")
            }
            Self::UnknownInternal(s) => {
                write!(f, "Internal not understood: {s:?}")
            }
        }
    }
}
impl std::error::Error for ParseActionError {}

impl fmt::Display for Action {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::Export(name) => write!(f, "export:{}", name),
            Self::Internal(internal) => internal.fmt(f),
            Self::Job(job_ref) => write!(f, "job:{}", job_ref),
        }
    }
}
impl FromStr for Action {
    type Err = ParseActionError;
    fn from_str(s: &str) -> Result<Self, ParseActionError> {
        if let Some((_, cat, con)) = regex_captures!(r#"^(\w+)\s*:\s*(.+)$"#, s) {
            match cat {
                "export" => Ok(Self::Export(con.into())),
                "internal" => {
                    // this prefix is optional
                    if let Ok(internal) = Internal::from_str(con) {
                        Ok(Self::Internal(internal))
                    } else {
                        Err(ParseActionError::UnknownInternal(con.to_string()))
                    }
                }
                "job" => Ok(Self::Job(con.into())),
                _ => Err(ParseActionError::UnknowCategory(cat.to_string())),
            }
        } else if let Ok(internal) = Internal::from_str(s) {
            Ok(Self::Internal(internal))
        } else {
            Err(ParseActionError::UnknownAction(s.to_string()))
        }
    }
}

impl From<Internal> for Action {
    fn from(i: Internal) -> Self {
        Self::Internal(i)
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
        serializer.serialize_str(&self.to_string())
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
        Action::Internal(Internal::Help),
        Action::Internal(Internal::Scroll(ScrollCommand::MilliPages(1500))),
        Action::Internal(Internal::Scroll(ScrollCommand::MilliPages(-500))),
        Action::Internal(Internal::Scroll(ScrollCommand::MilliPages(-2000))),
        Action::Export("my export".to_string()),
    ];
    for action in actions {
        println!("action: {}", action.to_string());
        assert_eq!(action.to_string().parse(), Ok(action));
    }
}
