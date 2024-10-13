use {
    crate::*,
    anyhow::*,
    serde::{
        Deserialize,
        Serialize,
    },
    std::process::ExitStatus,
};

/// what we get from the execution of a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
    /// a trustable report with errors and warnings computed
    Report(Report),
    /// we don't have a proper report
    Failure(Failure),
    /// not yet computed
    None,
}

impl CommandResult {
    pub fn build(
        output: CommandOutput,
        exit_status: Option<ExitStatus>,
        analyzer: Analyzer,
        mission: &Mission,
    ) -> Result<Self> {
        let lines = &output.lines;
        let error_code = exit_status.and_then(|s| s.code()).filter(|&c| c != 0);
        let mut report = analyzer.build_report(lines, mission)?;
        debug!("report stats: {:?}", &report.stats);
        if let Some(error_code) = error_code {
            if report.stats.errors + report.stats.test_fails == 0 {
                // report shows no error while the command exe reported
                // an error, so the report can't be trusted
                let suggest_backtrace = report.suggest_backtrace;
                return Ok(Self::Failure(Failure {
                    error_code,
                    output,
                    suggest_backtrace,
                }));
            }
        }
        report.output = output;
        // report looks valid
        Ok(Self::Report(report))
    }

    pub fn output(&self) -> Option<&CommandOutput> {
        match self {
            Self::Report(report) => Some(&report.output),
            Self::Failure(failure) => Some(&failure.output),
            Self::None => None,
        }
    }

    pub fn report(&self) -> Option<&Report> {
        match self {
            Self::Report(report) => Some(report),
            _ => None,
        }
    }

    pub fn suggest_backtrace(&self) -> bool {
        match self {
            Self::Report(report) => report.suggest_backtrace,
            Self::Failure(failure) => failure.suggest_backtrace,
            _ => false,
        }
    }

    /// return true when the report has been computed and there's been no
    /// error, warning, or test failures
    pub fn is_success(&self) -> bool {
        match self {
            Self::Report(report) => {
                report.stats.errors + report.stats.warnings + report.stats.test_fails == 0
            }
            _ => false,
        }
    }

    pub fn reverse(&mut self) {
        match self {
            Self::Report(report) => {
                report.reverse();
            }
            Self::Failure(failure) => {
                failure.output.reverse();
            }
            Self::None => {}
        }
    }
    pub fn lines_len(&self) -> usize {
        match self {
            Self::Report(report) => report.lines.len(),
            Self::Failure(failure) => failure.output.lines.len(),
            Self::None => 0,
        }
    }
}
