use {
    crate::*,
    anyhow::Result,
    std::{
        collections::HashSet,
        io,
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
            debug!("cmd_line={:?}", &cmd_line);
            let line_analysis = LineAnalysis::from(cmd_line);
            debug!("line_analysis={:?}", &line_analysis);
            let line_type = line_analysis.line_type;
            let mut line = Line {
                item_idx: 0, // will be filled later
                line_type,
                content: cmd_line.content.clone(),
            };
            match cmd_line.origin {
                CommandStream::StdErr => {
                    match line_type {
                        LineType::Title(Kind::Sum) => {
                            // we're not interested in this section
                            cur_err_kind = None;
                        }
                        LineType::Title(kind) => {
                            cur_err_kind = Some(kind);
                        }
                        _ => {}
                    }
                    match cur_err_kind {
                        Some(Kind::Warning) => warnings.push(line),
                        Some(Kind::Error) => errors.push(line),
                        _ => {} // before warnings and errors, or in a sum
                    }
                }
                CommandStream::StdOut => {
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
                            } else if is_in_out_fail {
                                fails.push(line);
                            }
                        }
                        (LineType::Title(Kind::Sum), None) => {
                            // we're not interested in this section
                            cur_err_kind = None;
                            is_in_out_fail = false;
                        }
                        (LineType::BacktraceSuggestion, _) => {
                            suggest_backtrace = true;
                        }
                        _ => {
                            // TODO add normal if not broken with blank line
                            warn!("unexpected line: {:#?}", &line);
                        }
                    }
                }
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
        Ok(Report {
            lines,
            stats,
            suggest_backtrace,
            output: CommandOutput::default(),
        })
    }
    /// export the report in a file
    pub fn write_to<W: io::Write>(
        &self,
        w: &mut W,
        mission: &Mission,
    ) -> Result<(), io::Error> {
        let mut last_cat = "???";
        for line in &self.lines {
            match line.line_type {
                LineType::Title(Kind::Warning) => {
                    last_cat = "warning";
                }
                LineType::Title(Kind::Error) => {
                    last_cat = "error";
                }
                LineType::Title(Kind::TestFail) => {
                    last_cat = "test";
                }
                _ => {}
            }
            if let Some(location) = line.location_path(mission) {
                writeln!(w, "{} {}", last_cat, location.to_string_lossy())?;
            }
        }
        debug!("exported locations");
        Ok(())
    }
}
