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
    last_test_key: Option<String>,
}

impl LineAnalyzer for NextestLineAnalyzer {
    fn analyze_line(
        &mut self,
        cmd_line: &CommandOutputLine,
    ) -> LineAnalysis {
        let content = &cmd_line.content;
        if let Some(key) = self.stdx_section_key(content) {
            return LineAnalysis::title_key(Kind::TestFail, key);
        }
        if let Some((key, pass)) = as_test_result(content) {
            self.last_test_key = Some(key.clone());
            return LineAnalysis::test_result(key, pass);
        }
        if is_canceling(content) {
            return LineAnalysis::of_type(LineType::SectionEnd);
        }
        if is_error_test_run_failed(content) {
            return LineAnalysis::of_type(LineType::Garbage);
        }
        if is_location_v3(content) {
            return LineAnalysis::of_type(LineType::Location);
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
impl NextestLineAnalyzer {
    fn stdx_section_key(
        &self,
        content: &TLine,
    ) -> Option<String> {
        self.stdx_section_key_v1(content)
            .or_else(|| self.stdx_section_key_v2(content))
            .or_else(|| self.stdx_section_key_v3(content))
    }
    /// Return the key when the line is like "--- STD(OUT|ERR): somekey ---"
    fn stdx_section_key_v1(
        &self,
        content: &TLine,
    ) -> Option<String> {
        let mut strings = content.strings.iter();
        let first = strings.next()?;
        if !regex_is_match!(r"^--- STD(OUT|ERR):\s*", &first.raw) {
            return None;
        }
        extract_key_after_crate_name_v1(strings)
    }
    /// Return the key when the line is like "──── STD(OUT|ERR): cratename some::key"
    fn stdx_section_key_v2(
        &self,
        content: &TLine,
    ) -> Option<String> {
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
    /// Return the last test key when the line is like "──── std(out|err)"
    /// For nextest 0.9.95+ (see https://github.com/Canop/bacon/issues/350)
    ///
    fn stdx_section_key_v3(
        &self,
        content: &TLine,
    ) -> Option<String> {
        let mut strings = content.strings.iter().skip_while(|ts| ts.is_blank());
        let ts = strings.next()?;
        if ts.csi != CSI_ERROR || !(ts.raw != "stderr" || ts.raw != "stdout") {
            return None;
        }
        let ts = strings.next()?;
        if !ts.is_blank() {
            return None;
        }
        let ts = strings.next()?;
        if ts.csi != CSI_ERROR || ts.raw != "───" {
            return None;
        }
        if strings.next().is_some() {
            return None;
        }
        self.last_test_key.clone()
    }
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

fn is_location_v3(content: &TLine) -> bool {
    let mut strings = content.strings.iter().skip_while(|ts| ts.is_blank());
    let Some(ts) = strings.next() else {
        return false;
    };
    if ts.csi != CSI_ERROR {
        return false;
    }
    regex_is_match!(
        r#"^\s*thread '.+' panicked at [^:\s'"]+:\d+:\d+:$"#,
        &ts.raw
    )
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
    let Some(first) = content.strings.first() else {
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
fn test_stdx_section_key_v1() {
    let analyzer = NextestLineAnalyzer::default();
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
        analyzer.stdx_section_key_v1(&content),
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
        analyzer.stdx_section_key_v1(&content),
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
