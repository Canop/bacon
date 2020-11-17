use crate::*;

/// result of the "parsing" of the line
#[derive(Debug, Clone)]
pub struct LineAnalysis {
    pub line_type: LineType,
    pub key: Option<String>,
}

impl From<&CommandOutputLine> for LineAnalysis {
    fn from(cmd_line: &CommandOutputLine) -> Self {
        let content = &cmd_line.content;
        let mut key = None;
        let line_type = match cmd_line.origin {
            CommandStream::StdOut => {
                // there's currently a not understood bug preventing us from getting
                // style information in stdout.
                if cmd_line.content.is_blank() {
                    LineType::Normal
                } else if let Some(content) = cmd_line.content.if_unstyled() {
                    if let Some((k, r)) = as_test_result(content) {
                        key = Some(k.to_string());
                        LineType::TestResult(r)
                    } else if let Some(k) = as_fail_result_title(content) {
                        key = Some(k.to_string());
                        LineType::Title(Kind::TestFail)
                    } else {
                        LineType::Normal
                    }
                } else {
                    warn!("unexpected styled stdout: {:#?}", &cmd_line); // wait ? there's no more the bug ?
                    LineType::Normal // unexpected styled content
                }
            }
            CommandStream::StdErr => {
                if let (Some(ts1), Some(ts2)) = (content.strings.get(0), content.strings.get(1)) {
                    match (ts1.csi.as_ref(), ts1.raw.as_ref(), ts2.csi.as_ref(), ts2.raw.as_ref()) {
                        (crate::CSI_BOLD_RED, "error", CSI_BOLD, r2) if r2.starts_with(": aborting due to") => {
                            LineType::Title(Kind::Sum)
                        }
                        (crate::CSI_BOLD_RED, r1, CSI_BOLD, _) if r1.starts_with("error") => {
                            LineType::Title(Kind::Error)
                        }
                        (crate::CSI_BOLD_YELLOW, "warning", _, r2) if is_n_warnings_emitted(&r2) => {
                            LineType::Title(Kind::Sum)
                        }
                        (crate::CSI_BOLD_YELLOW, "warning", _, _) => LineType::Title(Kind::Warning),
                        ("", r1, crate::CSI_BOLD_BLUE, "--> ") if is_spaces(r1) => LineType::Location,
                        _ => LineType::Normal,
                    }
                } else {
                    LineType::Normal // empty line
                }

            }
        };
        LineAnalysis {
            line_type,
            key,
        }
    }
}

fn is_spaces(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_whitespace())
}

/// check if the string starts with something like ": 15 warnings emitted"
fn is_n_warnings_emitted(s: &str) -> bool {
    let mut tokens = s.split_ascii_whitespace();
    if let Some(":") = tokens.next() {
        if let Some(t) = tokens.next() {
            if t.parse::<usize>().is_err() {
                return false;
            }
            if let Some(t) = tokens.next() {
                if t != "warnings" && t != "warning" {
                    return false;
                }
                if let Some(t) = tokens.next() {
                    if t.starts_with("emitted") {
                        return true;
                    }
                }
            }
        }
    }
    false
}


/// return Some when the line is the non detailled
/// result of a test, for example
///  "test str_fit::fitting_count_tests::test_count_fitting ... FAILED"
/// or
///  "test wrap::wrap_tests::check_space_removing ... ok"
fn as_test_result(s: &str) -> Option<(&str, bool)> {
    let mut tokens = s.split_ascii_whitespace();
    match (tokens.next(), tokens.next(), tokens.next(), tokens.next()) {
        (Some("test"), Some(key), Some("..."), Some("ok")) => Some((key, true)),
        (Some("test"), Some(key), Some("..."), Some("FAILED")) => Some((key, false)),
        _ => None,
    }
}

/// return Some(key) when the line is like this:
/// "---- key stdout ----"
fn as_fail_result_title(s: &str) -> Option<&str> {
    let mut tokens = s.split_ascii_whitespace();
    match (tokens.next(), tokens.next(), tokens.next(), tokens.next()) {
        (Some("----"), Some(key), Some("stdout"), Some("----")) => Some(key),
        _ => None,
    }
}
