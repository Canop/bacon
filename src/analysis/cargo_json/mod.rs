use {
    super::*,
    crate::{
        analysis::standard,
        *,
    },
    anyhow::Result,
    cargo_metadata::{
        Message,
        diagnostic::Diagnostic,
    },
};

/// An analyzer able to read the output
/// of `cargo check --message-format=json-diagnostic-rendered-ansi`
///
/// In order to be useful, this analyzer would have to read more than
/// just the 'rendered' field of the diagnostic. It would have to read
/// for example the spans, detect what's a suggestion, etc.
/// See https://github.com/Canop/bacon/issues/249
///
/// There are so many problems with this approach, though, that I'm
/// not sure this is worth it.
#[derive(Debug, Default)]
pub struct CargoJsonAnalyzer {
    lines: Vec<CommandOutputLine>,
}

impl Analyzer for CargoJsonAnalyzer {
    fn start(
        &mut self,
        _mission: &Mission,
    ) {
        self.lines.clear();
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
                let line = TLine::from_tty(&format!("Error parsing JSON: {}", err));
                let cmd_line = CommandOutputLine {
                    content: line,
                    origin: cmd_line.origin,
                };
                command_output.push(cmd_line);
            }
        }
    }

    fn build_report(&mut self) -> Result<Report> {
        let line_analyzer = standard::StandardLineAnalyzer {};
        standard::build_report(&self.lines, line_analyzer)
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
            Message::CompilerArtifact(_) => {}
            Message::CompilerMessage(compiler_message) => {
                self.receive_diagnostic(compiler_message.message, origin, command_output);
            }
            Message::BuildScriptExecuted(_) => {}
            Message::BuildFinished(_) => {}
            Message::TextLine(_) => {}
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
        let Diagnostic {
            //message,
            //code,
            //level,
            //spans,
            children,
            //suggestion,
            rendered,
            ..
        } = diagnostic;
        if let Some(rendered) = rendered {
            for line in rendered.trim().lines() {
                let content = TLine::from_tty(line);
                let cmd_line = CommandOutputLine { content, origin };
                command_output.push(cmd_line.clone());
                self.lines.push(cmd_line);
            }
        }
        for child in children {
            self.receive_diagnostic(child, origin, command_output);
        }
    }
}
