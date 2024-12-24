use {
    crate::{
        analysis::standard::StandardLineAnalyzer,
        *,
    },
    lazy_regex::*,
};

const CSI_TITLE: &str = "\u{1b}[35;1m";
const CSI_PASS: &str = "\u{1b}[32;1m";
const CSI_ERROR: &str = "\u{1b}[31;1m";

#[derive(Debug, Default)]
pub struct NextestLineAnalyzer {
    default_analyzer: StandardLineAnalyzer,
}

impl LineAnalyzer for NextestLineAnalyzer {
    fn analyze_line(
        &self,
        cmd_line: &CommandOutputLine,
    ) -> LineAnalysis {
        let content = &cmd_line.content;
        if let Some(key) = title_key(content).or_else(|| title_key_v2(content)) {
            return LineAnalysis::title_key(Kind::TestFail, key);
        }
        if let Some((key, pass)) = as_test_result(content) {
            return LineAnalysis::test_result(key, pass);
        }
        if is_canceling(content) {
            return LineAnalysis::of_type(LineType::SectionEnd);
        }
        if is_error_test_run_failed(content) {
            return LineAnalysis::of_type(LineType::Garbage);
        }
        if let Some(content) = cmd_line.content.if_unstyled() {
            if regex_is_match!(r"^running \d+ tests?$", content) {
                return LineAnalysis::of_type(LineType::Garbage);
            }
            if content == "------------" {
                return LineAnalysis::of_type(LineType::SectionEnd);
            }
            if content == "────────────" {
                return LineAnalysis::of_type(LineType::SectionEnd);
            }
        }
        // compilation warnings and errors are still composed with the standard cargo tool
        self.default_analyzer.analyze_line(cmd_line)
    }
}

fn title_key(content: &TLine) -> Option<String> {
    title_key_v1(content).or_else(|| title_key_v2(content))
}
/// Return the key when the line is like "--- STD(OUT|ERR): somekey ---"
fn title_key_v1(content: &TLine) -> Option<String> {
    let mut strings = content.strings.iter();
    let first = strings.next()?;
    if !regex_is_match!(r"^--- STD(OUT|ERR):\s*", &first.raw) {
        return None;
    }
    extract_key_after_crate_name_v1(strings)
}
/// Return the key when the line is like "──── STD(OUT|ERR): cratename some::key"
fn title_key_v2(content: &TLine) -> Option<String> {
    let mut strings = content.strings.iter();
    let ts = strings.next()?;
    if ts.csi != CSI_ERROR || ts.raw != "────" {
        return None;
    }
    let ts = strings.find(|ts| ts.raw != " ")?;
    if !regex_is_match!(r"^STD(OUT|ERR):\s*", &ts.raw) {
        return None;
    }
    extract_key_after_crate_name_v2(strings)
}

fn extract_key_after_crate_name_v1(mut strings: std::slice::Iter<'_, TString>) -> Option<String> {
    let _ = strings.next(); // skip blank
    let mut key = String::new();
    for s in &mut strings {
        if s.csi.is_empty() {
            continue;
        }
        if s.raw == " ---" || s.csi == CSI_TITLE {
            break;
        }
        key.push_str(&s.raw);
    }
    if strings.next().is_some() {
        return None;
    }
    if key.is_empty() { None } else { Some(key) }
}
fn extract_key_after_crate_name_v2(mut strings: std::slice::Iter<'_, TString>) -> Option<String> {
    // skip blank and crate name
    let ts = strings.find(|ts| ts.csi != CSI_TITLE && ts.raw != " ")?;
    let mut key = String::new();
    key.push_str(&ts.raw);
    for s in &mut strings {
        if s.csi.is_empty() || s.csi == CSI_TITLE {
            break;
        }
        key.push_str(&s.raw);
    }
    if key.is_empty() { None } else { Some(key) }
}

fn is_error_test_run_failed(content: &TLine) -> bool {
    let mut strings = content.strings.iter();
    let (Some(first), Some(second), None) = (strings.next(), strings.next(), strings.next()) else {
        return false;
    };
    first.csi == CSI_ERROR
        && first.raw.trim() == "error"
        && second.raw.trim() == ": test run failed"
}

fn is_canceling(content: &TLine) -> bool {
    let Some(first) = content.strings.get(0) else {
        return false;
    };
    first.csi == CSI_ERROR && first.raw.trim() == "Canceling"
}

/// return the key and whether the tests passes, when the line is a test
/// result (like "    PASS [   0.003s] bacon tests::failing_test3")
///
/// In the future, we might want to return the duration too.
fn as_test_result(content: &TLine) -> Option<(String, bool)> {
    as_test_result_v1(content).or_else(|| as_test_result_v2(content))
}
fn as_test_result_v1(content: &TLine) -> Option<(String, bool)> {
    let mut strings = content.strings.iter();
    let first = strings.next()?;
    let pass = match (first.csi.as_str(), first.raw.trim()) {
        (CSI_PASS, "PASS") => true,
        (CSI_ERROR, "FAIL") => false,
        _ => return None,
    };
    let _duration = match strings.next() {
        Some(s) if s.csi.is_empty() => s.raw.trim(),
        _ => return None,
    };
    let key = extract_key_after_crate_name_v1(strings)?;
    Some((key, pass))
}
/// return the key and whether the tests passes, when the line is a test
/// result (like "    PASS [   0.003s] bacon tests::failing_test3")
///
/// In the future, we might want to return the duration too.
fn as_test_result_v2(content: &TLine) -> Option<(String, bool)> {
    let mut strings = content.strings.iter();
    let first = strings.next()?;
    let pass = match (first.csi.as_str(), first.raw.trim()) {
        (CSI_PASS, "PASS") => true,
        (CSI_ERROR, "FAIL") => false,
        _ => return None,
    };
    let _duration = match strings.next() {
        Some(s) if s.csi.is_empty() => s.raw.trim(),
        _ => return None,
    };
    let key = extract_key_after_crate_name_v1(strings)?;
    Some((key, pass))
}

#[test]
fn test_title_key_v1() {
    let content = TLine {
        strings: vec![
            TString::new("\u{1b}[35;1m", "--- STDOUT:              bacon-test"),
            TString::new("", " "),
            TString::new("\u{1b}[36m", "tests::"),
            TString::new("\u{1b}[34;1m", "failing_test3"),
            TString::new("\u{1b}[35;1m", " ---"),
        ],
    };
    assert_eq!(
        title_key_v1(&content),
        Some("tests::failing_test3".to_string())
    );
    let content = TLine {
        strings: vec![
            TString::new("\u{1b}[31;1m", "--- STDERR:              bacon"),
            TString::new("", " "),
            TString::new("\u{1b}[36m", "analysis::nextest_analyzer::"),
            TString::new("\u{1b}[34;1m", "test_as_test_result"),
            TString::new("\u{1b}[31;1m", " ---"),
        ],
    };
    assert_eq!(
        title_key_v1(&content),
        Some("analysis::nextest_analyzer::test_as_test_result".to_string())
    );
}

#[test]
fn test_canceling() {
    let content = TLine {
        strings: vec![
            TString::new("\u{1b}[31;1m", "   Canceling"),
            TString::new("", " due to "),
            TString::new("\u{1b}[31;1m", "test failure"),
            TString::new("", ": "),
            TString::new("\u{1b}[1m", "1"),
            TString::new("", " test still running"),
        ],
    };
    assert_eq!(is_canceling(&content), true);
}

#[test]
fn test_as_test_result() {
    let content = TLine {
        strings: vec![
            TString::new("\u{1b}[32;1m", "        PASS"),
            TString::new("", " [   0.003s] "),
            TString::new("\u{1b}[35;1m", "bacon"),
            TString::new("", " "),
            TString::new("\u{1b}[36m", "analysis::nextest_analyzer::test_canceling"),
        ],
    };
    assert_eq!(
        as_test_result(&content),
        Some((
            "analysis::nextest_analyzer::test_canceling".to_string(),
            true
        ))
    );
}

#[test]
fn test_recognize_test_run_failed() {
    let content = TLine {
        strings: vec![
            TString::new("\u{1b}[31;1m", "error"),
            TString::new("", ": test run failed"),
        ],
    };
    assert!(is_error_test_run_failed(&content));
}
