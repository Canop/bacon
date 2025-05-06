use lazy_regex::*;

/// A command to focus on the diagnostics related
/// to a specific file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FocusFileCommand {
    pub file: String,
}

impl FocusFileCommand {
    /// Return the action description to show in doc/help
    pub fn doc(&self) -> String {
        format!("focus file {}", self.file)
    }
    /// Tell whether the location should be focused
    pub fn matches(
        &self,
        location: &str,
    ) -> bool {
        let Some((_, file, _line, _col)) = regex_captures!(r"^([^:]+)(:\d+)?(:\d+)?$", location)
        else {
            return false;
        };
        file.ends_with(&self.file)
    }
}
