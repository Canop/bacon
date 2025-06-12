//! Analyzer for swift build.
//!
//! # Config
//!
//! A sample bacon.toml:
//!
//! ```toml
//! default_job = "swift_build"
//!
//! [keybindings]
//! b = "job:swift_build"
//!
//! [jobs.swift_build]
//! command = ["swift", "build"]
//! watch = ["Sources"]
//! need_stdout = true
//! analyzer = "swift_build"
//! ```
//!
//! # Caveats
//!
//! This analyzer processes warnings, but `swift build` is incremental by default, and so warnings
//! will not be consistent. Consider treating warnings as errors if this is a problem.

use crate::{
    Analyzer,
    CommandOutputLine,
    ItemAccumulator,
    Kind,
    LineType,
    TLine,
    TString,
    burp,
};

use super::parse_swift_location;

#[derive(Debug, Default)]
pub struct SwiftBuildAnalyzer {
    lines: Vec<CommandOutputLine>,
}

impl Analyzer for SwiftBuildAnalyzer {
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
            if let Some(diagnostic) = recognize_swift_diagnostic(&line.content) {
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
            } else {
                items.push_line(LineType::Normal, line.content.clone());
            }
        }

        Ok(items.report())
    }
}

struct SwiftDiagnostic<'a> {
    level: Kind,
    path: &'a str,
    line: &'a str,
    column: &'a str,
    message: Vec<TString>,
}

fn recognize_swift_diagnostic(line: &TLine) -> Option<SwiftDiagnostic> {
    // Look for Swift format: path:line:column: (error|warning): message
    let content = line.if_unstyled()?;

    if let Some(error_pos) = content.find(": error: ") {
        let location_part = &content[..error_pos];
        let message_part = &content[error_pos + ": error: ".len()..];

        if let Some((path, line_num, column)) = parse_swift_location(location_part) {
            return Some(SwiftDiagnostic {
                level: Kind::Error,
                path,
                line: line_num,
                column,
                message: vec![TString::new("", message_part)],
            });
        }
    } else if let Some(warning_pos) = content.find(": warning: ") {
        let location_part = &content[..warning_pos];
        let message_part = &content[warning_pos + ": warning: ".len()..];

        if let Some((path, line_num, column)) = parse_swift_location(location_part) {
            return Some(SwiftDiagnostic {
                level: Kind::Warning,
                path,
                line: line_num,
                column,
                message: vec![TString::new("", message_part)],
            });
        }
    }

    None
}
