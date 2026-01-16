use {
    super::{biome, cargo_json, cpp, eslint, go, nextest, python, standard, swift, typescript},
    crate::*,
    schemars::JsonSchema,
    serde::{Deserialize, Serialize},
};

/// A stateless operator building a report from a list of command output lines.
///
/// Implementation routing will probably change at some point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AnalyzerRef {
    #[default]
    Standard,
    Biome,
    CargoJson,
    Cpp,
    CppDoctest,
    Eslint,
    Go,
    Nextest,
    PythonPytest,
    PythonRuff,
    PythonUnittest,
    SwiftBuild,
    SwiftLint,
    Typescript,
}

impl AnalyzerRef {
    pub fn create_analyzer(self) -> Box<dyn Analyzer> {
        match self {
            Self::Standard => Box::new(standard::StandardAnalyzer::default()),
            Self::Biome => Box::new(biome::BiomeAnalyzer::default()),
            Self::CargoJson => Box::new(cargo_json::CargoJsonAnalyzer::default()),
            Self::Cpp => Box::new(cpp::CppAnalyzer::default()),
            Self::CppDoctest => Box::new(cpp::CppDoctestAnalyzer::default()),
            Self::Eslint => Box::new(eslint::EslintAnalyzer::default()),
            Self::Go => Box::new(go::GoAnalyzer::default()),
            Self::Nextest => Box::new(nextest::NextestAnalyzer::default()),
            Self::PythonPytest => Box::new(python::pytest::PytestAnalyzer::default()),
            Self::PythonRuff => Box::new(python::ruff::RuffAnalyzer::default()),
            Self::PythonUnittest => Box::new(python::unittest::UnittestAnalyzer::default()),
            Self::SwiftBuild => Box::new(swift::build::SwiftBuildAnalyzer::default()),
            Self::SwiftLint => Box::new(swift::lint::SwiftLintAnalyzer::default()),
            Self::Typescript => Box::new(typescript::TypescriptAnalyzer::default()),
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
    );

    fn build_report(&mut self) -> anyhow::Result<Report>;
}
