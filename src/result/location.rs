use {
    lazy_regex::*,
    std::{
        path::PathBuf,
        str::FromStr,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    pub path: PathBuf,
    pub line: usize,
    pub column: Option<usize>,
}

impl FromStr for Location {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((_, path, line, column)) = regex_captures!(r#"^([^:\s]+):(\d+)(?:\:(\d+))?$"#, s,)
        else {
            return Err("invalid location format");
        };
        let line = line.parse().map_err(|_| "invalid line number")?; // too many digits
        let column = column.parse().ok();
        Ok(Self {
            path: PathBuf::from(path),
            line,
            column,
        })
    }
}
