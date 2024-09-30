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
    pub key: Option<String>,
}

impl LineAnalysis {
    pub fn of_type(line_type: LineType) -> Self {
        Self {
            line_type,
            key: None,
        }
    }
    pub fn normal() -> Self {
        Self::of_type(LineType::Normal)
    }
    pub fn title_key(
        kind: Kind,
        key: String,
    ) -> Self {
        Self {
            line_type: LineType::Title(kind),
            key: Some(key),
        }
    }
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
