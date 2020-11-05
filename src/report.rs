use {
    crate::*,
    anyhow::Result,
    std::{io::BufRead, path::Path},
};

/// the usable content of cargo watch's output,
/// lightly analyzed
#[derive(Debug)]
pub struct Report {
    pub lines: Vec<Line>,
    pub stats: Stats,
}

impl Report {
    pub fn try_from(stderr: &[u8]) -> Result<Report> {
        // we first accumulate warnings and errors in separate vectors
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut cur_kind = None;
        for line in stderr.lines() {
            let content = line?;
            let line_type = LineType::from(&content);
            match line_type {
                LineType::End => {
                    // we're not interested in what follows
                    break;
                }
                LineType::Title(kind) => {
                    cur_kind = Some(kind);
                }
                _ => {}
            }
            let line = Line {
                item_idx: 0, // will be filled later
                line_type,
                content,
            };
            match cur_kind {
                Some(Kind::Warning) => warnings.push(line),
                Some(Kind::Error) => errors.push(line),
                None => {} // before warnings and errors
            }
        }
        // we now build a common vector, with errors first
        let mut lines = errors;
        lines.append(&mut warnings);
        // and we assign the indexes
        let mut item_idx = 0;
        for line in &mut lines {
            if matches!(line.line_type, LineType::Title(_)) {
                item_idx += 1;
            }
            line.item_idx = item_idx;
        }
        // we compute the stats at end because some lines may
        // have been read but not added (at start or end)
        Ok(Report {
            stats: Stats::from(&lines),
            lines,
        })
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
        Report::try_from(&output.stderr)
    }
}
