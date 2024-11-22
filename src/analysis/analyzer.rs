use {
    anyhow::Result,
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
            Self::PythonUnittest => Box::new(python::unittest::PythonUnittestAnalyzer::default()),
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

}
