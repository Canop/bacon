use {
    crate::*,
    lazy_regex::*,
    serde::{de, Deserialize, Deserializer},
    std::{
        fmt,
        str::FromStr,
    },
};

/// an action that can be mapped to a key
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Internal(Internal),
    Job(JobRef),
}

#[derive(Debug)]
pub enum ParseActionError {
    WrongFormat,
    UnknowCategory(String),
    UnknownInternal(String),
}

impl fmt::Display for ParseActionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::WrongFormat => {
                write!(f, "Wrong format. Expected <category>:<name>")
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
            if cat == "internal" {
                if let Ok(internal) = Internal::from_str(con) {
                    Ok(Self::Internal(internal))
                } else {
                    Err(ParseActionError::UnknownInternal(con.to_string()))
                }
            } else if cat == "job" {
                Ok(Self::Job(con.into()))
            } else {
                Err(ParseActionError::UnknowCategory(cat.to_string()))
            }
        } else {
            Err(ParseActionError::WrongFormat)
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
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}
