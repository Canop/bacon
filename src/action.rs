use {
    crate::*,
    lazy_regex::*,
    serde::{
        Deserialize,
        Deserializer,
        de,
    },
    std::{
        fmt,
        str::FromStr,
    },
};

/// An action that can be mapped to a key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Export(String),
    Internal(Internal),
    Job(JobRef),
}

#[derive(Debug)]
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

impl FromStr for Action {
    type Err = ParseActionError;
    fn from_str(s: &str) -> Result<Self, ParseActionError> {
        if let Some((_, cat, con)) = regex_captures!(r#"^(\w+)\s*:\s*(\S+)$"#, s) {
            match cat {
                "export" => Ok(Self::Export(con.into())),
                "internal" => {
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

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}
