use {
    crate::*,
    anyhow::*,
    std::process::ExitStatus,
};

/// Dedicated to a mission, the report maker receives the command
/// output lines and builds a report at end, complete with starts.
pub struct ReportMaker {
    ignored_lines_patterns: Option<Vec<LinePattern>>,
    analyzer: Box<dyn Analyzer>,
}

impl ReportMaker {
    pub fn new(mission: &Mission) -> Self {
        let ignored_lines_patterns = mission.ignored_lines_patterns().cloned();
        let analyzer_ref = mission.analyzer();
        let analyzer = analyzer_ref.create_analyzer();
        Self {
            ignored_lines_patterns,
            analyzer,
        }
    }

    pub fn start(
        &mut self,
        mission: &Mission,
    ) {
        self.analyzer.start(mission);
    }

    pub fn receive_line(
        &mut self,
        cmd_line: CommandOutputLine,
        command_output: &mut CommandOutput,
    ) {
        if let Some(patterns) = self.ignored_lines_patterns.as_ref() {
            let raw_line = cmd_line.content.to_raw(); // FIXME could be made more efficient
            if patterns.iter().any(|p| p.raw_line_is_match(&raw_line)) {
                debug!("ignoring line: {}", &raw_line);
                return;
            }
        }
        self.analyzer.receive_line(cmd_line, command_output);
    }

    pub fn build_report(&mut self) -> Result<Report> {
        self.analyzer.build_report()
    }

    pub fn build_result(
        &mut self,
        output: CommandOutput,
        exit_status: Option<ExitStatus>,
    ) -> Result<CommandResult> {
        let report = self.analyzer.build_report()?;
        let result = CommandResult::build(output, exit_status, report)?;
        Ok(result)
    }
}
