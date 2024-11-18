//! An analyzer for eslint ( https://eslint.org/ )
use {
    super::*,
    crate::*,
    lazy_regex::*,
};

const CSI_LOCATION_PATH: &str = "\u{1b}[4m";
const CSI_LINE_COL: &str = "\u{1b}[2m";
const CSI_ERROR: &str = "\u{1b}[31m";
const CSI_WARNING: &str = "\u{1b}[33m";
const CSI_SUM: &str = "\u{1b}[31m\u{1b}[1m";


#[derive(Debug, Default)]
pub struct EslintLineAnalyzer;

impl LineAnalyzer for EslintLineAnalyzer {
    fn analyze_line(
        &self,
        cmd_line: &CommandOutputLine,
    ) -> LineAnalysis {
        if is_line_col_error_line(&cmd_line.content) {
            LineAnalysis {
                line_type: LineType::Title(Kind::Error),
                key: None,
            }
        } else if is_line_col_warning_line(&cmd_line.content) {
            LineAnalysis {
                line_type: LineType::Title(Kind::Warning),
                key: None,
            }
        } else if get_location_path(&cmd_line.content).is_some() {
            LineAnalysis {
                line_type: LineType::Location,
                key: None,
            }
        } else if is_sum(&cmd_line.content) {
            LineAnalysis {
                line_type: LineType::Title(Kind::Sum),
                key: None,
            }
        } else {
            LineAnalysis {
                line_type: LineType::Normal,
                key: None,
            }
        }
    }
}


/// Return true when the line is like
///    "67:52  error  Unnecessary escape character: \/  no-useless-escape"
fn is_line_col_error_line(content: &TLine) -> bool {
    let mut strings = content.strings.iter();
    let (Some(first), Some(second), Some(third), Some(fourth)) = (
        strings.next(),
        strings.next(),
        strings.next(),
        strings.next(),
    ) else {
        return false;
    };
    first.is_blank()
        && second.csi == CSI_LINE_COL
        && regex_is_match!(r"^\d+:\d+$", &second.raw)
        && third.is_blank()
        && fourth.csi == CSI_ERROR
        && fourth.raw == "error"
}

/// Return true when the line is like
///    "67:52  warning bla bla"
fn is_line_col_warning_line(content: &TLine) -> bool {
    let mut strings = content.strings.iter();
    let (Some(first), Some(second), Some(third), Some(fourth)) = (
        strings.next(),
        strings.next(),
        strings.next(),
        strings.next(),
    ) else {
        return false;
    };
    first.is_blank()
        && second.csi == CSI_LINE_COL
        && regex_is_match!(r"^\d+:\d+$", &second.raw)
        && third.is_blank()
        && fourth.csi == CSI_WARNING
        && fourth.raw == "warning"
}

fn is_sum(content: &TLine) -> bool {
    let mut strings = content.strings.iter();
    let Some(first) = strings.next() else {
        return false;
    };
    if !(first.csi == CSI_SUM
        && regex_is_match!(r"^✖ \d+ problems \(\d+ errors, \d+ warnings\)$", &first.raw))
    {
        return false;
    }
    for string in strings {
        if !string.is_blank() {
            return false;
        }
    }
    true
}
pub fn get_location_path(content: &TLine) -> Option<String> {
    let mut strings = content.strings.iter();
    let first = strings.next()?;
    if first.csi != CSI_LOCATION_PATH {
        return None;
    }
    // trying to recognize a path, I might make some wrong assumptions here,
    // especially for windows...
    if !regex_is_match!(r"^\s*/\S+\.\w+s\s*$", &first.raw) {
        return None;
    }
    Some(first.raw.to_string())
}

pub fn cleaned_tline(content: &TLine) -> TLine {
    let mut tline = TLine::default();
    let mut last_is_blank = true;
    for ts in &content.strings {
        if ts.csi == CSI_LINE_COL && regex_is_match!(r"^\d+:\d+$", &ts.raw) {
            continue; // useless line:col at start of title
        }
        let raw = regex_replace_all!(r"\s+", &ts.raw, " ").to_string();
        let is_blank = raw.trim().is_empty();
        if !(is_blank && last_is_blank) {
            tline.strings.push(TString::new(&ts.csi, raw));
        }
        last_is_blank = is_blank;
    }
    tline
}

