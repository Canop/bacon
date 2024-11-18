use {
    crate::*,
    serde::{
        Deserialize,
        Serialize,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisExport {
    #[serde(default)]
    pub analyzer: AnalyzerRef,
    pub result: CommandResultKind,
    pub lines: Vec<LineAnalysisExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineAnalysisExport {
    pub line: CommandOutputLine,
    pub analysis: LineAnalysis,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandResultKind {
    Report,
    Failure,
}

impl AnalysisExport {
    pub fn build(
        analyzer: AnalyzerRef,
        cmd_result: &CommandResult,
    ) -> Option<Self> {
        let mut lines = Vec::new();
        let (result, cmd_output_lines) = match cmd_result {
            CommandResult::Report(report) => (CommandResultKind::Report, &report.output.lines),
            CommandResult::Failure(failure) => (CommandResultKind::Failure, &failure.output.lines),
            CommandResult::None => {
                return None;
            }
        };
        for line in cmd_output_lines {
            let analysis = analyzer.analyze_line(line);
            lines.push(LineAnalysisExport {
                line: line.clone(),
                analysis,
            });
        }
        Some(Self {
            analyzer,
            result,
            lines,
        })
    }
}
