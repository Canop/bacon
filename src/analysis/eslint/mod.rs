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

pub fn analyze_line(cmd_line: &CommandOutputLine) -> LineAnalysis {
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

/// Build a report from the output of eslint
///
/// The main specificity of eslint is that the path of a file with error is given
/// before the errors of the file, each error coming with the line and column
/// in the file.
pub fn build_report(
    cmd_lines: &[CommandOutputLine],
    line_analyzer: Analyzer,
    mission: &Mission,
) -> anyhow::Result<Report> {
    let ignore_patterns = mission.ignored_lines_patterns();
    let mut items = ItemAccumulator::default();
    let mut last_location_path = None;
    for cmd_line in cmd_lines {
        if let Some(patterns) = ignore_patterns {
            let raw_line = cmd_line.content.to_raw();
            if patterns.iter().any(|p| p.raw_line_is_match(&raw_line)) {
                debug!("ignoring line: {}", &raw_line);
                continue;
            }
        }
        let line_analysis = line_analyzer.analyze_line(cmd_line);
        let line_type = line_analysis.line_type;
        match line_type {
            LineType::Garbage => {
                continue;
            }
            LineType::Title(kind) => {
                items.start_item(kind);
            }
            LineType::Normal => {}
            LineType::Location => {
                let path = get_location_path(&cmd_line.content);
                if let Some(path) = path {
                    last_location_path = Some(path);
                    continue;
                } else {
                    warn!("unconsistent line parsing");
                }
            }
            _ => {}
        }
        items.push_line(line_type, cleaned_tline(&cmd_line.content));
        if matches!(line_type, LineType::Title(_)) {
            // We just added the title, we must now add the location
            // As it's something which isn't present in eslint output, we're free
            // to choose the format we want so we're choosing the BURP one
            let line_col = cmd_line.content.strings.get(1).map(|s| s.raw.as_ref());
            let Some(line_col) = line_col else {
                warn!("unconsistent line parsing");
                continue;
            };
            let Some(location_path) = last_location_path.as_ref() else {
                warn!("no location given before error");
                continue;
            };
            items.push_line(
                LineType::Location,
                burp::location_line(location_path, line_col),
            );
        }
    }
    let lines = items.lines();
    let stats = Stats::from(&lines);
    debug!("stats: {:#?}", &stats);
    let report = Report {
        lines,
        stats,
        suggest_backtrace: false,
        output: Default::default(),
        failure_keys: Vec::new(),
    };
    Ok(report)
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
fn get_location_path(content: &TLine) -> Option<String> {
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

fn cleaned_tline(content: &TLine) -> TLine {
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
