use {
    serde::{de, Deserialize, Deserializer},
    std::{
        fmt,
        str::FromStr,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JobRef {
    Default,
    Initial,
    Previous,
    Concrete(ConcreteJobRef),
}

/// A "concrete" job ref is one which can be used from the start, without
/// referring to the job stack
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConcreteJobRef {
    Name(String),
    Alias(String),
}

impl Default for ConcreteJobRef {
    fn default() -> Self {
        Self::Name("check".to_string())
    }
}

impl fmt::Display for ConcreteJobRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Alias(alias) => write!(f, "alias:{alias}"),
            Self::Name(name) => write!(f, "{name}"),
        }
    }
}

impl FromStr for ConcreteJobRef {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("empty job name")
        } else {
            Ok(s.into())
        }
    }
}
impl<'de> Deserialize<'de> for ConcreteJobRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl From<&str> for ConcreteJobRef {
    fn from(str_entry: &str) -> Self {
        if let Some(alias) = str_entry.strip_prefix("alias:") {
            ConcreteJobRef::Alias(alias.to_string())
        } else {
            ConcreteJobRef::Name(str_entry.to_string())
        }
    }
}

impl fmt::Display for JobRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Initial => write!(f, "initial"),
            Self::Previous => write!(f, "previous"),
            Self::Concrete(concrete) => concrete.fmt(f),
        }
    }
}

impl From<&str> for JobRef {
    fn from(name: &str) -> Self {
        match name {
            "default" => Self::Default,
            "initial" => Self::Initial,
            "previous" => Self::Previous,
            _ => Self::Concrete(ConcreteJobRef::from(name)),
        }
    }
}
