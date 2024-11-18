use {
    crate::*,
};


// an utility for those analyzers that can work with the LineAnalysis struct
pub trait LineAnalyzer {

    /// this function will disappear
    fn analyze_line(
        &self,
        line: &CommandOutputLine,
    ) -> LineAnalysis;

}
