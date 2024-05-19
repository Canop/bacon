use {
    crate::*,
    anyhow::Result,
    lazy_regex::*,
    std::{
        collections::HashSet,
        io,
        path::PathBuf,
    },
};

/// the usable content of cargo watch's output,
/// lightly analyzed
#[derive(Debug)]
pub struct Report {
    pub lines: Vec<Line>,
    pub stats: Stats,
    pub suggest_backtrace: bool,
    pub output: CommandOutput,
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
    /// compute the report from the lines of stdout and/or stderr of the
    /// `cargo` command.
    ///
    /// We assume errors and warnings come in the stderr stream while
    ///  test failures come in stdout
    pub fn from_lines(cmd_lines: &[CommandOutputLine]) -> Result<Report> {
        // we first accumulate warnings, test fails and errors in separate vectors
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        let mut fails = Vec::new();
        let mut failure_names = HashSet::new();
        let mut passed_tests = 0;
        let mut cur_err_kind = None; // the current kind among stderr lines
        let mut is_in_out_fail = false;
        let mut suggest_backtrace = false;
        for cmd_line in cmd_lines {
            let line_analysis = LineAnalysis::from(cmd_line);
            let line_type = line_analysis.line_type;
            let mut line = Line {
                item_idx: 0, // will be filled later
                line_type,
                content: cmd_line.content.clone(),
            };
            debug!(
                "{:?}> [{line_type:?}][{:?}]",
                cmd_line.origin, line_analysis.key
            );
            match (line_type, line_analysis.key) {
                (LineType::TestResult(r), Some(key)) => {
                    if r {
                        passed_tests += 1;
                    } else {
                        // we should receive the test failure section later,
                        // right now we just whitelist it
                        failure_names.insert(key);
                    }
                }
                (LineType::Title(Kind::TestFail), Some(key)) => {
                    if failure_names.contains(&key) {
                        failure_names.remove(&key);
                        line.content = TLine::failed(&key);
                        fails.push(line);
                        is_in_out_fail = true;
                        cur_err_kind = Some(Kind::TestFail);
                    } else {
                        warn!(
                            "unexpected test result failure_names={:?}, key={:?}",
                            &failure_names, &key,
                        );
                    }
                }
                (LineType::Normal, None) => {
                    if line.content.is_blank() && cur_err_kind != Some(Kind::TestFail) {
                        is_in_out_fail = false;
                    }
                    if is_in_out_fail {
                        fails.push(line);
                    } else {
                        match cur_err_kind {
                            Some(Kind::Warning) => warnings.push(line),
                            Some(Kind::Error) => errors.push(line),
                            _ => {}
                        }
                    }
                }
                (LineType::Title(Kind::Sum), None) => {
                    // we're not interested in this section
                    cur_err_kind = None;
                    is_in_out_fail = false;
                }
                (LineType::Title(kind), _) => {
                    cur_err_kind = Some(kind);
                    match cur_err_kind {
                        Some(Kind::Warning) => warnings.push(line),
                        Some(Kind::Error) => errors.push(line),
                        _ => {} // before warnings and errors, or in a sum
                    }
                }
                (LineType::BacktraceSuggestion, _) => {
                    suggest_backtrace = true;
                }
                (LineType::Location, _) => {
                    match cur_err_kind {
                        Some(Kind::Warning) => warnings.push(line),
                        Some(Kind::Error) => errors.push(line),
                        Some(Kind::TestFail) => fails.push(line),
                        _ => {} // before warnings and errors, or in a sum
                    }
                }
                _ => {}
            }
        }
        // for now, we only added the test failures for which there was an output.
        // We add the other ones
        for key in failure_names.drain() {
            fails.push(Line {
                item_idx: 0, // will be filled later
                line_type: LineType::Title(Kind::TestFail),
                content: TLine::failed(&key),
            });
            fails.push(Line {
                item_idx: 0,
                line_type: LineType::Normal,
                content: TLine::italic("no output".to_string()),
            });
        }
        // we now build a common vector, with errors first
        let mut lines = errors;
        lines.append(&mut fails);
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
        let mut stats = Stats::from(&lines);
        stats.passed_tests = passed_tests;
        debug!("stats: {:#?}", &stats);
        Ok(Report {
            lines,
            stats,
            suggest_backtrace,
            output: CommandOutput::default(),
        })
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

    /// export the report in a file
    pub fn write_to<W: io::Write>(
        &self,
        w: &mut W,
        mission: &Mission,
    ) -> Result<(), io::Error> {
        let mut last_kind = "???";
        let mut message = None;
        let format_has_context = mission.settings.export.line_format.contains("{context}");
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
            let exported = regex_replace_all!(
                r#"\{([^\s}]+)\}"#,
                &mission.settings.export.line_format,
                |_, key| {
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
                }
            );
            writeln!(w, "{}", exported)?;
        }
        debug!("exported locations");
        Ok(())
    }
}
