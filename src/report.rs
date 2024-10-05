use {
    crate::*,
    anyhow::Result,
    lazy_regex::*,
    serde::{
        Deserialize,
        Serialize,
    },
    std::{
        io,
        path::PathBuf,
    },
};

/// the usable content of cargo watch's output,
/// lightly analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub lines: Vec<Line>,
    pub stats: Stats,
    pub suggest_backtrace: bool,
    pub output: CommandOutput,
    pub failure_keys: Vec<String>,
}

impl Report {
    /// change the order of the lines so that items are in reverse order
    /// (but keep the order of lines of a given item)
    pub fn reverse(&mut self) {
        self.lines
            .sort_by_key(|line| std::cmp::Reverse(line.item_idx));
    }
    /// A successful report is one with nothing to tell: no warning,
    /// no error, no test failure
    pub fn is_success(
        &self,
        allow_warnings: bool,
        allow_failures: bool,
    ) -> bool {
        !(self.stats.errors != 0
            || (!allow_failures && self.stats.test_fails != 0)
            || (!allow_warnings && self.stats.warnings != 0))
    }

    /// Extract all the diagnostic context, that is all the normal lines
    /// which have the same item index as the given line.
    /// Those lines are taken without style and joined with an escaped newline.
    fn extract_raw_diagnostic_context(
        &self,
        line: &Line,
    ) -> String {
        self.lines
            .iter()
            .filter(|l| l.line_type == LineType::Normal && l.item_idx == line.item_idx)
            .map(|l| l.content.to_raw())
            .collect::<Vec<String>>()
            .join("\\n")
    }

    /// export the report in a file, as the "locations" format
    pub fn write_locations<W: io::Write>(
        &self,
        w: &mut W,
        mission: &Mission, // used to get the workspace root to normalize locations
        line_format: &str,
    ) -> Result<(), io::Error> {
        let mut last_kind = "???";
        let mut message = None;
        let format_has_context = line_format.contains("{context}");
        for line in &self.lines {
            match line.line_type {
                LineType::Title(Kind::Warning) => {
                    last_kind = "warning";
                    message = line.title_message();
                }
                LineType::Title(Kind::Error) => {
                    last_kind = "error";
                    message = line.title_message();
                }
                LineType::Title(Kind::TestFail) => {
                    last_kind = "test";
                    message = line.title_message();
                }
                _ => {}
            }
            let Some(location) = line.location() else {
                continue;
            };
            let (_, mut path, file_line, file_column) =
                regex_captures!(r#"^([^:\s]+):(\d+):(\d+)$"#, location)
                    .unwrap_or(("", location, "", ""));
            // we need to make sure the path is absolute
            let path_buf = PathBuf::from(path);
            let path_string;
            if path_buf.is_relative() {
                path_string = mission
                    .workspace_root
                    .join(path)
                    .to_string_lossy()
                    .to_string();
                path = &path_string;
            }
            let extracted_context;
            let context = if format_has_context {
                extracted_context = self.extract_raw_diagnostic_context(line);
                &extracted_context
            } else {
                ""
            };
            let exported = regex_replace_all!(r#"\{([^\s}]+)\}"#, line_format, |_, key| {
                match key {
                    "column" => file_column,
                    "context" => context,
                    "kind" => last_kind,
                    "line" => file_line,
                    "message" => message.unwrap_or(""),
                    "path" => path,
                    _ => {
                        debug!("unknown export key: {key:?}");
                        ""
                    }
                }
            });
            writeln!(w, "{}", exported)?;
        }
        debug!("exported locations");
        Ok(())
    }
}
