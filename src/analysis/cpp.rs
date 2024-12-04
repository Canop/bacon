use {
    crate::*,
    anyhow::Result,
    lazy_regex::regex_is_match,
};

#[derive(Debug, Default)]
pub struct CppAnalyzer {
    lines: Vec<CommandOutputLine>,
}

impl Analyzer for CppAnalyzer {
    fn start(
        &mut self,
        _mission: &Mission,
    ) {
        self.lines.clear()
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

fn build_report(lines: &[CommandOutputLine]) -> Result<Report> {
    let mut items = ItemAccumulator::default();
    let mut in_item = false;

    for line in lines {
        let (line_type, start, end) = if line.content.is_blank() {
            (LineType::Garbage, None, false)
        } else if let Some(content) = line.content.if_unstyled() {
            if content.contains("error: ") {
                (LineType::Title(Kind::Error), Some(Kind::Error), false)
            } else if content.contains("warning: ") {
                (LineType::Title(Kind::Warning), Some(Kind::Warning), false)
            } else if regex_is_match!(r"^ *\d* *\|", content) {
                (LineType::Normal, None, false)
            } else {
                (LineType::Normal, None, true)
            }
        } else if let (Some(_loc), Some(kind)) =
            (line.content.strings.first(), line.content.strings.get(1))
        {
            match kind.raw.as_str() {
                "error: " => (LineType::Title(Kind::Error), Some(Kind::Error), false),
                "warning: " => (LineType::Title(Kind::Warning), Some(Kind::Warning), false),
                _ => (LineType::Normal, None, false),
            }
        } else {
            (LineType::Normal, None, true)
        };
        if let Some(kind) = start {
            items.start_item(kind);
            in_item = true;
        } else if end {
            items.close_item();
            in_item = false;
        }

        if in_item {
            items.push_line(line_type, line.content.clone());
        }
    }

    let lines = items.lines();
    let stats = Stats::from(&lines);
    Ok(Report {
        lines,
        stats,
        suggest_backtrace: false,
        output: Default::default(),
        failure_keys: Vec::new(),
    })
}
