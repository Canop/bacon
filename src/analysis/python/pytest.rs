//! An analyzer for Python's Pytest test framework.
use {
    crate::*,
    anyhow::Result,
    lazy_regex::*,
};

#[derive(Debug, Default)]
pub struct PytestAnalyzer {
    lines: Vec<CommandOutputLine>,
}

enum PytLineFormat<'l> {
    H1(&'l str), // big title with `=`
    H2(&'l str), // smaller title with `_`
    Location { path: &'l str, line: &'l str },
    Other,
}
enum Section {
    Errors,
    Failures,
    Other,
}

impl Analyzer for PytestAnalyzer {
    fn start(
        &mut self,
        _mission: &Mission,
    ) {
        self.lines.clear();
    }

    fn receive_line(
        &mut self,
        line: CommandOutputLine,
        command_output: &mut CommandOutput,
    ) {
        self.lines.push(line.clone());
        command_output.push(line);
    }

    fn build_report(&mut self) -> Result<Report> {
        build_report(&self.lines)
    }
}

fn recognize_format(content: &str) -> PytLineFormat<'_> {
    regex_switch!(content,
        r"^(?:={2,39}) (?<title>.+) (?:={2,39})$" => PytLineFormat::H1(title),
        r"^(?:_{2,39}) (?<title>.+) (?:_{2,39})$" => PytLineFormat::H2(title),
        r"^file (?<path>\S+\.py), line (?<line>\d{1,8})$" => PytLineFormat::Location { path, line },
        r"^(?<path>\S+\.py):(?<line>\d{1,8})" => PytLineFormat::Location { path, line },
    )
    .unwrap_or(PytLineFormat::Other)
}

/// Build a report from the output of Python unittest
///
/// The main special thing here is transforming the location line in
/// a BURP location line.
pub fn build_report(cmd_lines: &[CommandOutputLine]) -> anyhow::Result<Report> {
    let mut current_section = Section::Other;
    let mut items = ItemAccumulator::default();
    let mut last_location_in_item = None; // to deduplicate locations
    for cmd_line in cmd_lines {
        let Some(content) = cmd_line.content.if_unstyled() else {
            continue; // right now we're not expecting styled output
        };
        let format = recognize_format(content);
        match format {
            PytLineFormat::H1(title) => {
                current_section = match title {
                    "ERRORS" => Section::Errors,
                    "FAILURES" => Section::Failures,
                    _ => Section::Other,
                };
                items.close_item();
            }
            PytLineFormat::H2(title) => match current_section {
                Section::Errors => {
                    items.push_error_title(burp::error_line(title));
                    last_location_in_item = None;
                }
                Section::Failures => {
                    items.push_failure_title(burp::failure_line(title));
                    last_location_in_item = None;
                }
                _ => {}
            },
            PytLineFormat::Location { path, line } => {
                if let Some(last_location) = last_location_in_item {
                    if last_location == (path, line) {
                        continue;
                    }
                }
                last_location_in_item = Some((path, line));
                items.push_line(
                    LineType::Location,
                    burp::location_line(format!("{path}:{line}")),
                );
            }
            PytLineFormat::Other => {
                items.push_line(LineType::Normal, cmd_line.content.clone());
            }
        }
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
