use {
    crate::*,
    anyhow::Result,
    lazy_regex::*,
    rustc_hash::FxHashSet,
    serde::{
        Deserialize,
        Serialize,
    },
    std::{
        collections::HashMap,
        io,
        path::PathBuf,
    },
};

/// the usable content of cargo watch's output,
/// lightly analyzed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub output: CommandOutput,
    pub lines: Vec<Line>,
    pub stats: Stats,
    pub suggest_backtrace: bool,
    pub failure_keys: Vec<String>,
    /// the exports that the analyzers have done, by name
    pub analyzer_exports: HashMap<String, String>,

    pub has_passed_tests: bool,

    pub dismissed_items: usize,
    pub dismissed_lines: Vec<Line>,
}

impl Report {
    pub fn new(lines: Vec<Line>) -> Self {
        let stats = Stats::from(&lines);
        Self {
            output: Default::default(),
            lines,
            suggest_backtrace: false,
            failure_keys: Vec::new(),
            analyzer_exports: Default::default(),
            has_passed_tests: false,
            stats,
            dismissed_items: 0,
            dismissed_lines: Vec::new(),
        }
    }

    pub fn lines_changed(&mut self) {
        self.stats = Stats::from(&self.lines);
    }

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

    pub fn remove_item(
        &mut self,
        item_idx: usize,
    ) {
        self.lines.retain(|line| line.item_idx != item_idx);
    }

    pub fn has_dismissed_items(&self) -> bool {
        self.dismissed_items > 0
    }

    pub fn focus_file(
        &mut self,
        ffc: &FocusFileCommand,
    ) {
        let focused_idxs = self
            .lines
            .iter()
            .filter(|line| {
                line.location()
                    .is_some_and(|location| ffc.matches(location))
            })
            .map(|line| line.item_idx)
            .collect::<FxHashSet<_>>();
        let is_reversed = self.lines.first().map_or(0, |line| line.item_idx)
            > self.lines.last().map_or(0, |line| line.item_idx);
        let cmp = |a: &Line, b: &Line| match (
            focused_idxs.contains(&a.item_idx),
            focused_idxs.contains(&b.item_idx),
        ) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.item_idx.cmp(&b.item_idx),
        };
        if is_reversed {
            self.lines.sort_by(|a, b| cmp(b, a));
        } else {
            self.lines.sort_by(cmp);
        }
    }

    pub fn top_item_idx(&self) -> Option<usize> {
        self.lines.first().map(|line| line.item_idx)
    }

    pub fn item_location(
        &self,
        item_idx: usize,
    ) -> Option<&str> {
        info!("looking for location of item {item_idx}");
        self.lines
            .iter()
            .find(|line| line.item_idx == item_idx && line.location().is_some())
            .and_then(|line| line.location())
        //.and_then(|loc| loc.parse().ok())
    }

    pub fn item_diag_type(
        &self,
        item_idx: usize,
    ) -> Option<&str> {
        info!("looking for diag_type of item {item_idx}");
        for line in &self.lines {
            if line.item_idx != item_idx {
                continue;
            }
            let diag_type = line.diag_type();
            if diag_type.is_some() {
                return diag_type;
            }
        }
        None
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
        let mut current_item_idx = 0usize;
        for line in &self.lines {
            match line.line_type {
                LineType::Title(Kind::Warning) => {
                    last_kind = "warning";
                    message = line.title_message();
                    current_item_idx = line.item_idx;
                }
                LineType::Title(Kind::Error) => {
                    last_kind = "error";
                    message = line.title_message();
                    current_item_idx = line.item_idx;
                }
                LineType::Title(Kind::TestFail) => {
                    last_kind = "test";
                    message = line.title_message();
                    current_item_idx = line.item_idx;
                }
                _ => {}
            }
            let Some(location) = line.location() else {
                continue;
            };
            let (_, path, file_line, mut file_column) =
                regex_captures!(r#"^([^:\s]+):(\d+)(?:\:(\d+))?$"#, location)
                    .unwrap_or(("", location, "", ""));
            // we need to make sure the path is absolute
            let path_buf = PathBuf::from(path);
            let path_buf = mission.make_absolute(path_buf);
            let path = path_buf.to_string_lossy().to_string();
            let extracted_context;
            let context = if format_has_context {
                extracted_context = self.extract_raw_diagnostic_context(line);
                &extracted_context
            } else {
                ""
            };
            if file_column.is_empty() {
                file_column = "1"; // by default, first column in file
            }
            let item_idx_str = current_item_idx.to_string();
            let job_name = mission.concrete_job_ref.badge_label();
            let exported = regex_replace_all!(r#"\{([^\s}]+)\}"#, line_format, |_, key| {
                match key {
                    "column" => file_column,
                    "context" => context,
                    "item-idx" => &item_idx_str,
                    "job" =>  &job_name,
                    "kind" => last_kind,
                    "line" => file_line,
                    "message" => message.unwrap_or(""),
                    "path" => &path,
                    _ => {
                        debug!("unknown export key: {key:?}");
                        ""
                    }
                }
            });
            writeln!(w, "{exported}")?;
        }
        debug!("exported locations");
        Ok(())
    }
    pub fn can_scope_tests(&self) -> bool {
        self.has_passed_tests && self.stats.test_fails > 0
    }
}
