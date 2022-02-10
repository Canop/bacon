use {
    std::fmt,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JobRef {
    Default,
    Initial,
    Previous,
    Name(String), // should be neither "initial", "default" or "previous"
}

impl fmt::Display for JobRef {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Initial => write!(f, "initial"),
            Self::Previous => write!(f, "previous"),
            Self::Name(name) => write!(f, "{name:?}"),
        }
    }
}

impl From<&str> for JobRef {
    fn from(name: &str) -> Self {
        match name {
            "default" => Self::Default,
            "initial" => Self::Initial,
            "previous" => Self::Previous,
            _ => Self::Name(name.to_string()),
        }
    }
}
