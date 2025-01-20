//! Utilities related to BURP
//! See <https://dystroy.org/blog/bacon-everything-roadmap/#introduce-burp>
use crate::*;

/// Make a BURP compliant location line
pub fn location_line<S: Into<String>>(location: S) -> TLine {
    let mut line = TLine::default();
    line.strings
        .push(TString::new("\u{1b}[1m\u{1b}[38;5;12m", "   --> "));
    line.strings.push(TString::new("", location));
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
/// Make a BURP compliant error line (title) from a [TString]
pub fn error_line_ts(error: &[TString]) -> TLine {
    let mut line = TLine::default();
    line.strings.push(TString::new(CSI_BOLD_RED, "error"));
    line.strings.push(TString::new("", ": "));
    line.strings.extend(error.iter().cloned());
    line
}
/// Make a BURP compliant warning line (title) from a [TString]
pub fn warning_line_ts(warning: &[TString]) -> TLine {
    let mut line = TLine::default();
    line.strings.push(TString::new(CSI_BOLD_YELLOW, "warning"));
    line.strings.push(TString::new("", ": "));
    line.strings.extend(warning.iter().cloned());
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
