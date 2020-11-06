use {
    crate::*,
    anyhow::Result,
    std::{
        io::BufRead,
        path::Path,
    },
};

/// the usable content of cargo watch's output,
/// lightly analyzed
#[derive(Debug)]
pub struct Report {
    pub lines: Vec<Line>,
    pub stats: Stats,
}

impl Report {
    /// compute the report from the sderr of `cargo watch`
    pub fn from_bytes(stderr: &[u8]) -> Result<Report> {
        let mut lines = Vec::new();
        for line in stderr.lines() {
            lines.push(line?);
        }
        Self::from_err_lines(lines)
    }

    /// compute the report from the lines of stderr of `cargo watch`
    pub fn from_err_lines(err_lines: Vec<String>) -> Result<Report> {
        // we first accumulate warnings and errors in separate vectors
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut cur_kind = None;
        for content in err_lines {
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

    /// build (and doesn't call) the external command which
    /// will be used for the report
    pub fn get_command(root_dir: &Path, use_clippy: bool) -> Command {
        let sub_command = if use_clippy { "clippy" } else { "check" };
        let mut command = Command::new("cargo");
        command
            .arg(sub_command)
            .arg("--color")
            .arg("always")
            .current_dir(root_dir);
        command
    }

    /// compute the whole report in one go
    ///
    /// (not used today but might be in the future)
    pub fn compute(root_dir: &Path, use_clippy: bool) -> Result<Report> {
        let output = Self::get_command(root_dir, use_clippy)
            .output()?;
        Report::from_bytes(&output.stderr)
    }

}
