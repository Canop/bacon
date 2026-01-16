//! An analyzer for eslint ( <https://eslint.org/> )
mod eslint_line_analyzer;

use {
    super::*,
    crate::*,
    anyhow::Result,
    eslint_line_analyzer::*,
};

#[derive(Debug, Default)]
pub struct EslintAnalyzer {
    lines: Vec<CommandOutputLine>,
}

impl Analyzer for EslintAnalyzer {
    fn start(
        &mut self,
        _mission: &Mission,
    ) {
        self.lines.clear();
    }

    fn receive_line(
        &mut self,
        line: CommandOutputLine,
        command_output: &mut CommandOutput,
    ) {
        self.lines.push(line.clone());
        command_output.push(line);
    }

    fn build_report(&mut self) -> Result<Report> {
        Ok(build_report(&self.lines))
    }
}

/// Build a report from the output of eslint
///
/// The main specificity of eslint is that the path of a file with error is given
/// before the errors of the file, each error coming with the line and column
/// in the file.
pub fn build_report(cmd_lines: &[CommandOutputLine]) -> Report {
    let mut line_analyzer = EslintLineAnalyzer {};
    let mut items = ItemAccumulator::default();
    let mut last_location_path = None;
    for cmd_line in cmd_lines {
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
                }
                warn!("inconsistent line parsing");
            }
            _ => {}
        }
        items.push_line(line_type, cleaned_tline(&cmd_line.content));
        if matches!(line_type, LineType::Title(_)) {
            // We just added the title, we must now add the location
            // As it's something which isn't present in eslint output, we're free
            // to choose the format we want so we're choosing the BURP one
            let line_col: Option<&str> = cmd_line.content.strings.get(1).map(|s| s.raw.as_ref());
            let Some(line_col) = line_col else {
                warn!("inconsistent line parsing");
                continue;
            };
            let Some(location_path) = last_location_path.as_ref() else {
                warn!("no location given before error");
                continue;
            };
            items.push_line(
                LineType::Location,
                burp::location_line(format!("{location_path}:{line_col}")),
            );
        }
    }
    items.report()
}
