//! An analyzer for Python unittest
use {
    crate::*,
    lazy_regex::*,
};

pub fn analyze_line(cmd_line: &CommandOutputLine) -> LineAnalysis {
    // we're not expecting styled output for unittest (it's probable
    // some users use decorators, but I don't know those today)
    let Some(content) = cmd_line.content.if_unstyled() else {
        return LineAnalysis::normal();
    };
    regex_switch!(content,
        r"^FAIL:\s+\S+\s+\((?<key>.+)\)" => LineAnalysis::fail(key),
        r#"^\s+File ".+", line \d+"# => LineAnalysis::of_type(LineType::Location),
        "^={50,}$" => LineAnalysis::garbage(),
        "^-{50,}$" => LineAnalysis::garbage(),
        r"^Traceback \(most recent call last\)" => LineAnalysis::garbage(),
    )
    .unwrap_or_else(LineAnalysis::normal)
}

/// Build a report from the output of eslint
///
/// The main special thing here is transforming the location line in
/// a BURP location line.
pub fn build_report(
    cmd_lines: &[CommandOutputLine],
    line_analyzer: Analyzer,
    mission: &Mission,
) -> anyhow::Result<Report> {
    let ignore_patterns = mission.ignored_lines_patterns();
    let mut items = ItemAccumulator::default();
    let mut item_location_written = false;
    for cmd_line in cmd_lines {
        if let Some(patterns) = ignore_patterns {
            let raw_line = cmd_line.content.to_raw();
            if patterns.iter().any(|p| p.raw_line_is_match(&raw_line)) {
                debug!("ignoring line: {}", &raw_line);
                continue;
            }
        }
        let line_analysis = line_analyzer.analyze_line(cmd_line);
        let line_type = line_analysis.line_type;
        match line_type {
            LineType::Garbage => {
                continue;
            }
            LineType::Title(kind) => {
                items.start_item(kind);
                item_location_written = false;
            }
            LineType::Normal => {}
            LineType::Location => {
                if !item_location_written {
                    if let Some(content) = cmd_line.content.if_unstyled() {
                        // we rewrite the location as a BURP location
                        if let Some((_, path, line)) =
                            regex_captures!(r#"\s+File "(.+)", line (\d+)"#, content,)
                        {
                            items.push_line(LineType::Location, burp::location_line(path, line));
                            item_location_written = true;
                        } else {
                            warn!("unconsistent line parsing");
                        }
                        continue;
                    }
                }
            }
            _ => {}
        }
        items.push_line(line_type, cmd_line.content.clone());
    }
    let lines = items.lines();
    let stats = Stats::from(&lines);
    debug!("stats: {:#?}", &stats);
    let report = Report {
        lines,
        stats,
        suggest_backtrace: false,
        output: Default::default(),
        failure_keys: Vec::new(),
    };
    Ok(report)
}
