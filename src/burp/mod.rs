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
