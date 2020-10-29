

use {
    crate::*,
    anyhow::Result,
    std::{
        convert::TryFrom,
        io::{self, Read, BufRead},
    },
};


#[derive(Debug)]
pub struct Report {
    pub warnings: Vec<Item>,
    pub errors: Vec<Item>,
}

/// "warning" in bold yellow, followed by a bold colon
static WARNING: &'static str = "\u{1b}[0m\u{1b}[1m\u{1b}[33mwarning\u{1b}[0m\u{1b}[0m\u{1b}[1m: ";

/// "error" in bold red, followed by a bold colon
static ERROR: &'static str = "\u{1b}[0m\u{1b}[1m\u{1b}[38;5;9merror\u{1b}[0m\u{1b}[0m\u{1b}[1m: ";

impl Report {

    pub fn try_from(stderr: &Vec<u8>) -> Result<Report> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut cur_event: Option<Item> = None;
        for line in stderr.lines() {
            let line = line?;
            //println!("line {}", line);
            let new_kind = if line.starts_with(WARNING) {
                Some(Kind::Warning)
            } else if line.starts_with(ERROR) {
                Some(Kind::Error)
            } else {
                None
            };
            //println!("  event type: {:?}", new_type);
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
            } else if let Some(ref mut event) = cur_event.as_mut() {
                event.lines.push(line);
            }
        }
        Ok(Report {
            warnings,
            errors,
        })
    }

    pub fn compute() -> Result<Report> {
        debug!("starting cargo check");
        let output = Command::new("cargo")
             .arg("check")
             .arg("--color")
             .arg("always")
             .output()
             .context("Failed to run_cargo_check")?;
        debug!("cargo check finished");
        debug!("status: {:?}", &output.status);
        // println!("stdout:");
        // io::stdout().write_all(&output.stdout)?;
        // println!("stderr:");
        // io::stderr().write_all(&output.stderr)?;
        let report = Report::try_from(&output.stderr)?;
        debug!("report: {} warnings and {} errors", report.warnings.len(), report.errors.len());
        Ok(report)
    }
}


