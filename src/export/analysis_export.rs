use {
    crate::*,
    serde::{
        Deserialize,
        Serialize,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisExport {
    pub lines: Vec<LineAnalysisExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineAnalysisExport {
    pub line: CommandOutputLine,
    pub analysis: LineAnalysis,
}

impl AnalysisExport {
    pub fn build(cmd_output_lines: &[CommandOutputLine]) -> Self {
        let mut lines = Vec::new();
        for line in cmd_output_lines {
            let analysis = LineAnalysis::from(line);
            lines.push(LineAnalysisExport {
                line: line.clone(),
                analysis,
            });
        }
        Self { lines }
    }
}
