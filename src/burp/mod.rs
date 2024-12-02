//! Utilities related to BURP
//! See https://dystroy.org/blog/bacon-everything-roadmap/#introduce-burp
use crate::*;

/// Make a BURP compliant location line
pub fn location_line(
    location_path: &str,
    line_col: &str,
) -> TLine {
    let mut line = TLine::default();
    line.strings
        .push(TString::new("\u{1b}[1m\u{1b}[38;5;12m", "   --> "));
    line.strings
        .push(TString::new("", format!("{}:{}", location_path, line_col)));
    line
}

/// Make a BURP compliant error line (title)
pub fn error_line(error: &str) -> TLine {
    let mut line = TLine::default();
    line.strings.push(TString::new(CSI_BOLD_RED, "error"));
    line.strings.push(TString::new("", ": "));
    line.strings.push(TString::new("", error.to_string()));
    line
}
/// Make a BURP compliant test failure line (title)
/// (this one isn't based on cargo)
pub fn failure_line(error: &str) -> TLine {
    let mut line = TLine::default();
    line.strings.push(TString::new(CSI_BOLD_YELLOW, "failure"));
    line.strings.push(TString::new("", ": "));
    line.strings.push(TString::new("", error.to_string()));
    line
}
