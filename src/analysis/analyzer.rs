use {
    anyhow::Result,
    super::{
        eslint,
        nextest,
        //python,
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
pub enum AnalyzerRef {
    #[default]
    Standard,
    Nextest,
    Eslint,
    PythonUnittest,
}

impl AnalyzerRef {

    pub fn create_analyzer(
        self,
    ) -> Box<dyn Analyzer> {
        match self {
            Self::Standard => Box::new(standard::StandardAnalyzer::default()),
            Self::Nextest => Box::new(nextest::NextestAnalyzer::default()),
            Self::Eslint => Box::new(eslint::EslintAnalyzer::default()),
            _ => {
                todo!()
            }
        }
    }

}

pub trait Analyzer {

    fn start(
        &mut self,
        mission: &Mission,
    );

    fn receive_line(
        &mut self,
        line: CommandOutputLine,
        command_output: &mut CommandOutput,
    ); // return line type ? analysis ?

    fn build_report(
        &mut self,
    ) -> Result<Report>;


    //pub fn analyze_line(
    //    self,
    //    line: &CommandOutputLine,
    //) -> LineAnalysis {
    //    match self {
    //        Self::Eslint => eslint::analyze_line(line),
    //        Self::Standard => standard::analyze_line(line),
    //        Self::Nextest => nextest::analyze_line(line),
    //        Self::PythonUnittest => python::unittest::analyze_line(line),
    //    }
    //}
    //pub fn build_report(
    //    &self,
    //    cmd_lines: &[CommandOutputLine],
    //    mission: &Mission,
    //) -> anyhow::Result<Report> {
    //    match self {
    //        Self::Eslint => eslint::build_report(cmd_lines, *self, mission),
    //        Self::PythonUnittest => python::unittest::build_report(cmd_lines, *self, mission),
    //        Self::Standard | Self::Nextest => {
    //            // nextest analyzis simply uses the standard report building
    //            standard::build_report(cmd_lines, *self, mission)
    //        }
    //    }
    //}
}
