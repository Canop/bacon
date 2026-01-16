mod standard_line_analyser;
mod standard_report_building;

pub use {
    standard_line_analyser::StandardLineAnalyzer,
    standard_report_building::build_report,
};

use {
    crate::*,
    anyhow::Result,
};

#[derive(Debug, Default)]
pub struct StandardAnalyzer {
    lines: Vec<CommandOutputLine>,
}

impl Analyzer for StandardAnalyzer {
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
        let line_analyzer = StandardLineAnalyzer {};
        Ok(build_report(&self.lines, line_analyzer))
    }
}
