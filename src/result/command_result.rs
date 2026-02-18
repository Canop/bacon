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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        exit_status: ExitStatus,
        mut report: Report,
    ) -> Result<Self> {
        let error_code = exit_status.code().filter(|&c| c != 0);
        debug!("report stats: {:?}", &report.stats);
        if let Some(error_code) = error_code {
            let stats = &report.stats;
            if stats.errors + stats.test_fails + stats.warnings == 0 {
                // Report shows no error while the command exe reported
                // an error, so the report can't be trusted.
                // Note that some tools return an error on warnings (eg
                // miri), some don't.
                let suggest_backtrace = report.suggest_backtrace;
                return Ok(Self::Failure(Failure {
                    error_code,
                    output,
                    suggest_backtrace,
                }));
            }
        }
        report.output = output;
        report.error_code = error_code;
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

    pub fn report_mut(&mut self) -> Option<&mut Report> {
        match self {
            Self::Report(report) => Some(report),
            _ => None,
        }
    }

    pub fn suggest_backtrace(&self) -> bool {
        match self {
            Self::Report(report) => report.suggest_backtrace,
            Self::Failure(failure) => failure.suggest_backtrace,
            Self::None => false,
        }
    }

    /// return true when the report has been computed and there's been no
    /// error, warning, or test failures
    ///
    /// This is different from the `is_success` that a mission can compute
    /// from a report using its own settings (eg `allow_warnings`)
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
