use {
    super::{
        eslint,
        nextest,
        python,
        standard,
    },
    crate::*,
    serde::{
        Deserialize,
        Serialize,
    },
};

/// A stateless operator building a report from a list of command output lines.
///
/// Implementation routing will probably change at some point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Analyzer {
    #[default]
    Standard,
    Nextest,
    Eslint,
    PythonUnittest,
}

impl Analyzer {
    pub fn analyze_line(
        self,
        line: &CommandOutputLine,
    ) -> LineAnalysis {
        match self {
            Self::Eslint => eslint::analyze_line(line),
            Self::Standard => standard::analyze_line(line),
            Self::Nextest => nextest::analyze_line(line),
            Self::PythonUnittest => python::unittest::analyze_line(line),
        }
    }
    pub fn build_report(
        &self,
        cmd_lines: &[CommandOutputLine],
        mission: &Mission,
    ) -> anyhow::Result<Report> {
        match self {
            Self::Eslint => eslint::build_report(cmd_lines, *self, mission),
            Self::PythonUnittest => python::unittest::build_report(cmd_lines, *self, mission),
            Self::Standard | Self::Nextest => {
                // nextest analyzis simply uses the standard report building
                standard::build_report(cmd_lines, *self, mission)
            }
        }
    }
}
