use {
    lazy_regex::regex::Regex,
    serde::{
        Deserialize,
        Deserializer,
        de,
    },
    std::str::FromStr,
};

/// A pattern dedicated to line matching.
///
/// In the future, this may become more complex (eg filtering by style or origin)
#[derive(Debug, Clone)]
pub struct LinePattern {
    pub regex: Regex,
}

impl LinePattern {
    pub fn raw_line_is_match(
        &self,
        line: &str,
    ) -> bool {
        self.regex.is_match(line)
    }
}

impl FromStr for LinePattern {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(s).map_err(|e| format!("invalid regex: {}", e))?;
        Ok(Self { regex })
    }
}

impl<'de> Deserialize<'de> for LinePattern {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl PartialEq for LinePattern {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.regex.as_str() == other.regex.as_str()
    }
}
