use {
    crate::*,
    anyhow::Result,
    std::{io::BufRead, path::Path},
};

#[derive(Debug)]
pub struct Report {
    pub warnings: Vec<Item>,
    pub errors: Vec<Item>,
}

static STYLED_PREFIX: &str = "\u{1b}[0m";

/// "warning" in bold yellow, followed by a bold colon
static WARNING: &str = "\u{1b}[1m\u{1b}[33mwarning\u{1b}[0m\u{1b}[0m\u{1b}[1m: ";

/// "error" in bold red
static ERROR: &str = "\u{1b}[1m\u{1b}[38;5;9merror";

/// a "-->" in bold blue
static ARROW: &str = "\u{1b}[0m\u{1b}[0m\u{1b}[1m\u{1b}[38;5;12m--> \u{1b}[0m\u{1b}[0m";

impl Report {
    pub fn try_from(stderr: &[u8]) -> Result<Report> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut cur_event: Option<Item> = None;
        for line in stderr.lines() {
            let content = line?;
            let (new_kind, summary) = if let Some(styled) = content.strip_prefix(STYLED_PREFIX) {
                let styled = styled.trim();
                if styled.starts_with(WARNING) {
                    (Some(Kind::Warning), true)
                } else if styled.starts_with(ERROR) {
                    (Some(Kind::Error), true)
                } else if styled.starts_with(ARROW) {
                    debug!(" ---> arrow");
                    (None, true)
                } else {
                    (None, false)
                }
            } else {
                (None, false)
            };

            let line = Line { content, summary };
            if let Some(kind) = new_kind {
                if let Some(event) = cur_event.take() {
                    match event.kind {
                        Kind::Warning => warnings.push(event),
                        Kind::Error => errors.push(event),
                    }
                }
                cur_event = Some(Item {
                    kind,
                    lines: vec![line],
                });
            } else if let Some(event) = cur_event.as_mut() {
                event.lines.push(line);
            }
        }
        Ok(Report { warnings, errors })
    }

    pub fn compute(root_dir: &Path, use_clippy: bool) -> Result<Report> {
        let command = if use_clippy { "clippy" } else { "check" };
        debug!("starting cargo {}", command);
        let output = Command::new("cargo")
            .arg(command)
            .arg("--color")
            .arg("always")
            .current_dir(root_dir)
            .output()
            .with_context(|| format!("Failed to run cargo {}", command))?;
        debug!("cargo {} finished", command);
        debug!("status: {:?}", &output.status);
        let report = Report::try_from(&output.stderr)?;
        debug!(
            "report: {} warnings and {} errors",
            report.warnings.len(),
            report.errors.len()
        );
        Ok(report)
    }
}
