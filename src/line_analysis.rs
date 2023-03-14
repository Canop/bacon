use {
    crate::*,
    lazy_regex::*,
};

#[cfg(not(windows))]
const CSI_ERROR_BODY: &str = CSI_BOLD;
#[cfg(windows)]
const CSI_ERROR_BODY: &str = CSI_BOLD_WHITE;

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
                    } else if regex_is_match!("^failures:$", content) {
                        // this isn't very discriminant...
                        LineType::Title(Kind::Sum)
                    } else if regex_is_match!("^note: run with `RUST_BACKTRACE=", content) {
                        LineType::BacktraceSuggestion
                    } else {
                        LineType::Normal
                    }
                } else {
                    warn!("unexpected styled stdout: {:#?}", &cmd_line);
                    LineType::Normal // unexpected styled content
                }
            }
            CommandStream::StdErr => {
                if let (Some(title), Some(body)) = (content.strings.get(0), content.strings.get(1))
                {
                    match (
                        title.csi.as_ref(),
                        title.raw.as_ref(),
                        body.csi.as_ref(),
                        body.raw.as_ref(),
                    ) {
                        (CSI_BOLD_RED, "error", CSI_ERROR_BODY, body_raw)
                            if body_raw.starts_with(": aborting due to") =>
                        {
                            LineType::Title(Kind::Sum)
                        }
                        (CSI_BOLD_RED, title_raw, CSI_ERROR_BODY, _)
                            if title_raw.starts_with("error") =>
                        {
                            LineType::Title(Kind::Error)
                        }
                        #[cfg(not(windows))]
                        (CSI_BOLD_YELLOW, "warning", _, body_raw) => {
                            determine_warning_type(body_raw, content)
                        }
                        #[cfg(windows)]
                        (CSI_BOLD_YELLOW | CSI_BOLD_4BIT_YELLOW, "warning", _, body_raw) => {
                            determine_warning_type(body_raw, content)
                        }
                        ("", title_raw, CSI_BOLD_BLUE, "--> ") if is_spaces(title_raw) => {
                            debug!("LOCATION {:#?}", &content);
                            LineType::Location
                        }
                        _ => LineType::Normal,
                    }
                } else {
                    LineType::Normal // empty line
                }
            }
        };
        LineAnalysis { line_type, key }
    }
}

fn determine_warning_type(
    body_raw: &str,
    content: &TLine,
) -> LineType {
    if is_n_warnings_emitted(body_raw) || is_generated_n_warnings(content.strings.get(2)) {
        LineType::Title(Kind::Sum)
    } else {
        LineType::Title(Kind::Warning)
    }
}

fn is_spaces(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_whitespace())
}

/// check if the string starts with something like ": 15 warnings emitted"
fn is_n_warnings_emitted(s: &str) -> bool {
    regex_is_match!(r#"^: \d+ warnings? emitted"#, s)
}

fn is_generated_n_warnings(ts: Option<&TString>) -> bool {
    ts.map_or(false, |ts| {
        regex_is_match!(r#"generated \d+ warnings?$"#, &ts.raw)
    })
}
/// return Some when the line is the non detailled
/// result of a test, for example
///
///  "test str_fit::fitting_count_tests::test_count_fitting ... FAILED"
/// or
///  "test wrap::wrap_tests::check_space_removing ... ok"
/// or
///  "test src/lib.rs - (line 6) ... FAILED"
/// or
///  "test src/lib.rs - (line 10) ... ok"
/// or
///  "test src/mode.rs - mode::Mode::new (line 121) - compile ... FAILED"
/// (in this case, the " - compile" part isn't in the key, see #64)
/// or
///  "test tests::another - should panic ... FAILED"
/// (in this case, the " - should panic" part isn't in the key, see #95)
fn as_test_result(s: &str) -> Option<(&str, bool)> {
    regex_captures!(
        r#"^test\s+(.+?)(?: - should panic\s*)?(?: - compile\s*)?\s+...\s+(\w+)$"#,
        s
    )
    .and_then(|(_, key, outcome)| match outcome {
        "ok" => Some((key, true)),
        "FAILED" => Some((key, false)),
        other => {
            warn!("unrecognized doctest outcome: {:?}", other);
            None
        }
    })
}

/// return Some(key) when the line is like this:
/// "---- key stdout ----"
fn as_fail_result_title(s: &str) -> Option<&str> {
    regex_captures!(r#"^---- (.+) stdout ----$"#, s).map(|(_, key)| key)
}
