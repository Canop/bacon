use {
    crate::*,
    serde::{
        Deserialize,
        Serialize,
    },
};

/// result of the "parsing" of the line
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineAnalysis {
    pub line_type: LineType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl LineAnalysis {
    #[must_use]
    pub fn of_type(line_type: LineType) -> Self {
        Self {
            line_type,
            key: None,
        }
    }
    #[must_use]
    pub fn normal() -> Self {
        Self::of_type(LineType::Normal)
    }
    #[must_use]
    pub fn garbage() -> Self {
        Self::of_type(LineType::Garbage)
    }
    #[must_use]
    pub fn title_key(
        kind: Kind,
        key: String,
    ) -> Self {
        Self {
            line_type: LineType::Title(kind),
            key: Some(key),
        }
    }
    pub fn fail<S: Into<String>>(key: S) -> Self {
        Self {
            line_type: LineType::Title(Kind::TestFail),
            key: Some(key.into()),
        }
    }
    #[must_use]
    pub fn test_result(
        key: String,
        pass: bool,
    ) -> Self {
        Self {
            line_type: LineType::TestResult(pass),
            key: Some(key),
        }
    }
}
