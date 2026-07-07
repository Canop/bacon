mod cargo_json_export;

use {
    super::*,
    crate::{
        analysis::standard::build_report_from_analysis,
        *,
    },
    anyhow::Result,
    cargo_json_export::*,
    cargo_metadata::{
        Message,
        diagnostic::{
            Diagnostic,
            DiagnosticLevel,
        },
    },
    lazy_regex::*,
};

/// An analyzer able to read the output
/// of `cargo check --message-format=json-diagnostic-rendered-ansi`
///
/// In order to be useful, this analyzer would have to read more than
/// just the 'rendered' field of the diagnostic. It would have to read
/// for example the spans, detect what's a suggestion, etc.
/// See <https://github.com/Canop/bacon/issues/249>
///
/// There are so many problems with this approach, though, that I'm
/// not sure this is worth it.
#[derive(Default)]
pub struct CargoJsonAnalyzer {
    analysis: Vec<(LineAnalysis, TLine)>,
    exports: Vec<CargoJsonExport>,
}

impl Analyzer for CargoJsonAnalyzer {
    fn start(
        &mut self,
        mission: &Mission,
    ) {
        self.analysis.clear();
        self.exports.clear();
        for (name, export_settings) in &mission.settings.exports.exports {
            if export_settings.exporter == Exporter::Analyser {
                let export = CargoJsonExport::new(name.clone(), export_settings);
                self.exports.push(export);
            }
        }
    }

    fn receive_line(
        &mut self,
        cmd_line: CommandOutputLine,
        command_output: &mut CommandOutput,
    ) {
        let Some(content) = cmd_line.content.if_unstyled() else {
            return; // right now we're not expecting styled output
        };
        match serde_json::from_str::<Message>(content) {
            Ok(message) => {
                self.receive_cargo_message(message, cmd_line.origin, command_output);
            }
            Err(err) => {
                let line = TLine::from_tty(&format!("Error parsing JSON: {err}"));
                let cmd_line = CommandOutputLine {
                    content: line,
                    origin: cmd_line.origin,
                };
                command_output.push(cmd_line);
            }
        }
    }

    fn build_report(&mut self) -> Result<Report> {
        let mut report = build_report_from_analysis(self.analysis.drain(..));
        for export in self.exports.drain(..) {
            report.analyzer_exports.insert(export.name, export.export);
        }
        Ok(report)
    }
}

impl CargoJsonAnalyzer {
    fn receive_cargo_message(
        &mut self,
        message: Message,
        origin: CommandStream,
        command_output: &mut CommandOutput,
    ) {
        match message {
            Message::CompilerMessage(compiler_message) => {
                self.receive_diagnostic(compiler_message.message, origin, command_output);
            }
            _ => {
                // non exhaustive enum
            }
        }
    }
    fn receive_diagnostic(
        &mut self,
        diagnostic: Diagnostic,
        origin: CommandStream,
        command_output: &mut CommandOutput,
    ) {
        for export in &mut self.exports {
            export.receive_diagnostic(&diagnostic);
        }
        let Diagnostic {
            //message,
            //code,
            level,
            //spans,
            children,
            //suggestion,
            rendered,
            ..
        } = diagnostic;
        if let Some(rendered) = rendered {
            let mut line_type = match level {
                DiagnosticLevel::Error | DiagnosticLevel::Ice => LineType::Title(Kind::Error),
                DiagnosticLevel::Warning => LineType::Title(Kind::Warning),
                _ => LineType::Normal,
            };
            for line in rendered.trim().lines() {
                let content = TLine::from_tty(line);
                command_output.push(CommandOutputLine {
                    content: content.clone(),
                    origin,
                });
                if line_type == LineType::Normal {
                    let raw = content.to_raw();
                    if regex_is_match!(r":\d+:\d+", &raw) {
                        line_type = LineType::Location;
                    }
                }
                self.analysis
                    .push((LineAnalysis::of_type(line_type), content));
                line_type = LineType::Normal;
            }
        }
        for child in children {
            self.receive_diagnostic(child, origin, command_output);
        }
    }
}
