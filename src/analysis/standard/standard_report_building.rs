use {
    crate::*,
    rustc_hash::FxHashMap,
};

pub fn build_report<L: LineAnalyzer>(
    cmd_lines: &[CommandOutputLine],
    mut line_analyzer: L,
) -> anyhow::Result<Report> {
    #[derive(Debug, Default)]
    struct Failure {
        has_title: bool,
    }
    // we first accumulate warnings, test fails and errors in separate vectors
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut fails = Vec::new();
    let mut failures: FxHashMap<String, Failure> = Default::default();
    let mut passed_tests = 0;
    let mut cur_err_kind = None; // the current kind among stderr lines
    let mut is_in_out_fail = false;
    let mut suggest_backtrace = false;
    for cmd_line in cmd_lines {
        let line_analysis = line_analyzer.analyze_line(cmd_line);
        let line_type = line_analysis.line_type;
        let mut line = Line {
            item_idx: 0, // will be filled later
            line_type,
            content: cmd_line.content.clone(),
        };
        match (line_type, line_analysis.key) {
            (LineType::Garbage, _) => {
                continue;
            }
            (LineType::TestResult(r), Some(key)) => {
                if r {
                    passed_tests += 1;
                } else if !failures.contains_key(&key) {
                    // we should receive the test failure section later,
                    // right now we just whitelist it
                    failures.insert(key, Failure::default());
                }
            }
            (LineType::Title(Kind::TestFail), Some(key)) => {
                let failure = failures.entry(key.clone()).or_default();
                is_in_out_fail = true;
                cur_err_kind = Some(Kind::TestFail);
                if failure.has_title {
                    // we already have a title for this failure
                    // (for nextest, we have a title for stdout and one for stderr)
                    continue;
                }
                failure.has_title = true;
                line.content = TLine::failed(&key);
                fails.push(line);
            }
            (LineType::Normal, None) => {
                if line.content.is_blank() {
                    if cur_err_kind == Some(Kind::TestFail) {
                        // beautification: we remove some blank lines
                        if let Some(last) = fails.last() {
                            if last.line_type != LineType::Normal || last.content.is_blank() {
                                continue;
                            }
                        }
                    } else {
                        is_in_out_fail = false;
                    }
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
            (LineType::SectionEnd, None) => {
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
    for (key, failure) in &failures {
        // if we know of a failure but there was no content, we add some
        if failure.has_title {
            continue;
        }
        fails.push(Line {
            item_idx: 0, // will be filled later
            line_type: LineType::Title(Kind::TestFail),
            content: TLine::failed(key),
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
    let failure_keys = failures.keys().cloned().collect();
    let report = Report {
        lines,
        stats,
        suggest_backtrace,
        output: Default::default(),
        failure_keys,
        analyzer_exports: Default::default(),
    };
    Ok(report)
}
