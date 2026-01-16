mod nextest_line_analyser;

use {
    crate::*,
    anyhow::Result,
    nextest_line_analyser::NextestLineAnalyzer,
};

#[derive(Debug, Default)]
pub struct NextestAnalyzer {
    lines: Vec<CommandOutputLine>,
}

impl Analyzer for NextestAnalyzer {
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
        let line_analyzer = NextestLineAnalyzer::default();
        let report = crate::analysis::standard::build_report(&self.lines, line_analyzer);
        Ok(report)
    }
}
