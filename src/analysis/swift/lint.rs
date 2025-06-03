//! Analyzer for [`swiftlint`](https://github.com/realm/SwiftLint)
//!
//! # Config
//!
//! A sample bacon.toml:
//!
//! ```toml
//! [keybindings]
//! l = "job:swift_lint"
//!
//! [jobs.swift_lint]
//! wcommand = ["swiftlint", "lint", "--config", ".swiftlint.yml", "--strict"]
//! watch = ["Sources"]
//! need_stdout = true
//! analyzer = "swift_lint"
//! ```

use crate::{Analyzer, CommandOutputLine, ItemAccumulator, Kind, LineType, TLine, TString, burp};

use super::parse_swift_location;

#[derive(Debug, Default)]
pub struct SwiftLintAnalyzer {
    lines: Vec<CommandOutputLine>,
}

impl Analyzer for SwiftLintAnalyzer {
    fn start(
        &mut self,
        _: &crate::Mission,
    ) {
        self.lines.clear();
    }

    fn receive_line(
        &mut self,
        line: CommandOutputLine,
        command_output: &mut crate::CommandOutput,
    ) {
        self.lines.push(line.clone());
        command_output.push(line);
    }

    fn build_report(&mut self) -> anyhow::Result<crate::Report> {
        let mut items = ItemAccumulator::default();

        for line in &self.lines {
            if let Some(diagnostic) = recognize_swiftlint_diagnostic(&line.content) {
                let content = match diagnostic.level {
                    Kind::Error => burp::error_line_ts(&diagnostic.message),
                    Kind::Warning => burp::warning_line_ts(&diagnostic.message),
                    _ => unreachable!(),
                };

                items.start_item(diagnostic.level);
                items.push_line(LineType::Title(diagnostic.level), content);
                items.push_line(
                    LineType::Location,
                    burp::location_line(format!(
                        "{}:{}:{}",
                        diagnostic.path, diagnostic.line, diagnostic.column
                    )),
                );

                if let Some(rule) = diagnostic.rule {
                    items.push_line(LineType::Normal, TLine::from_raw(format!("Rule: {}", rule)));
                }
            } else {
                items.push_line(LineType::Normal, line.content.clone());
            }
        }

        Ok(items.report())
    }
}

struct SwiftLintDiagnostic<'a> {
    level: Kind,
    path: &'a str,
    line: &'a str,
    column: &'a str,
    message: Vec<TString>,
    rule: Option<&'a str>,
}

fn recognize_swiftlint_diagnostic(line: &TLine) -> Option<SwiftLintDiagnostic> {
    let content = line.if_unstyled()?;

    // Skip linting progress lines
    if content.starts_with("Linting '") || content.starts_with("Done linting!") {
        return None;
    }

    if let Some(error_pos) = content.find(": error: ") {
        let location_part = &content[..error_pos];
        let message_part = &content[error_pos + ": error: ".len()..];

        if let Some((path, line_num, column)) = parse_swift_location(location_part) {
            let (message, rule) = extract_message_and_rule(message_part);
            return Some(SwiftLintDiagnostic {
                level: Kind::Error,
                path,
                line: line_num,
                column,
                message: vec![TString::new("", message)],
                rule,
            });
        }
    } else if let Some(warning_pos) = content.find(": warning: ") {
        let location_part = &content[..warning_pos];
        let message_part = &content[warning_pos + ": warning: ".len()..];

        if let Some((path, line_num, column)) = parse_swift_location(location_part) {
            let (message, rule) = extract_message_and_rule(message_part);
            return Some(SwiftLintDiagnostic {
                level: Kind::Warning,
                path,
                line: line_num,
                column,
                message: vec![TString::new("", message)],
                rule,
            });
        }
    }

    None
}

fn extract_message_and_rule(message_part: &str) -> (&str, Option<&str>) {
    // Look for rule name in parentheses at the end: "message (rule_name)"
    if let Some(last_paren) = message_part.rfind('(') {
        if message_part.ends_with(')') {
            let message = message_part[..last_paren].trim();
            let rule = &message_part[last_paren + 1..message_part.len() - 1];
            return (message, Some(rule));
        }
    }
    (message_part, None)
}
