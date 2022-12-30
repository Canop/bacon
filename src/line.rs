use {
    crate::*,
    std::path::PathBuf,
};

/// A report line
#[derive(Debug)]
pub struct Line {
    /// the index among items
    /// (all lines having the same index belong to
    /// the same error, warning, or test item)
    pub item_idx: usize,

    pub line_type: LineType,

    pub content: TLine,
}

impl Line {
    /// Return the location as given by cargo (usually relative)
    pub fn location(&self) -> Option<&str> {
        match self.line_type {
            LineType::Location => {
                self.content.strings.get(2).map(|ts| ts.raw.as_str())
            }
            _ => None,
        }
    }
    /// Return the absolute path to the error/warning/test location
    pub fn location_path(&self, mission: &Mission) -> Option<PathBuf> {
        let location_path = self.location()?;
        let mut location_path = PathBuf::from(location_path);
        if !location_path.is_absolute() {
            location_path = mission.workspace_root.join(location_path);
        }
        Some(location_path)
    }
}

impl WrappableLine for Line {
    fn content(&self) -> &TLine {
        &self.content
    }
    fn prefix_cols(&self) -> usize {
        self.line_type.cols()
    }
}
