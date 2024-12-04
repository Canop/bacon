use {
    crate::*,
    anyhow::Result,
    std::borrow::Cow,
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

struct Diagnostic<'l> {
    level: Kind,
    path: &'l str,
    line: &'l str,
    column: &'l str,
    message: Cow<'l, [TString]>,
}

fn build_report(lines: &[CommandOutputLine]) -> Result<Report> {
    let mut items = ItemAccumulator::default();

    for line in lines {
        let diag = recognize_diagnostic(&line.content);
        if let Some(Diagnostic {
            level,
            path,
            line,
            column,
            message,
        }) = diag
        {
            let content = match level {
                Kind::Error => burp::error_line_ts(&message),
                Kind::Warning => burp::warning_line_ts(&message),
                _ => unreachable!(),
            };

            items.start_item(level);
            items.push_line(LineType::Title(level), content);
            items.push_line(
                LineType::Location,
                burp::location_line(format!("{path}:{line}:{column}")),
            );
        } else {
            items.push_line(LineType::Normal, line.content.clone())
        }
    }

    let lines = items.lines();
    let stats = Stats::from(&lines);
    info!("stats: {:#?}", &stats);
    Ok(Report {
        lines,
        stats,
        suggest_backtrace: false,
        output: Default::default(),
        failure_keys: Vec::new(),
    })
}

/// Expect that diagnostics look like the following:
///
/// ```text
/// [path/to/source:line:column:] [level:] [message...]
/// ```
///
/// ... where each part is contained within a potentially styled section.
fn recognize_diagnostic(line: &TLine) -> Option<Diagnostic> {
    let (pos, level, kind_end) = line.strings.iter().enumerate().find_map(|(idx, section)| {
        if let Some(found) = section.raw.find("error:") {
            let remaining = (found + "error: ".len()).min(section.raw.len());
            Some((idx, Kind::Error, remaining))
        } else if let Some(found) = section.raw.find("warning:") {
            let remaining = (found + "warning: ".len()).min(section.raw.len());
            Some((idx, Kind::Warning, remaining))
        } else {
            None
        }
    })?;

    let (path, loc_line, loc_col) = line.strings.iter().take(pos + 1).find_map(|section| {
        let mut it = section.raw.split(':');
        Some((it.next()?.trim(), it.next()?.trim(), it.next()?.trim()))
    })?;

    let message = if kind_end == 0 {
        // if the "warning:" section only contains the warning text,
        // take all remaining sections as the message
        Cow::Borrowed(line.strings.get(pos + 1..).unwrap_or_default())
    } else {
        // otherwise, we need to extract whatever is left in the type section
        // specifically:
        // - raw[kind_end..]       -> the rest of the type section
        // - strings[pos+1..] -> all remaining sections
        let level_section = &line.strings[pos];
        let first = TString {
            csi: level_section.csi.clone(),
            raw: String::from(level_section.raw[kind_end..].trim_start_matches(' ')),
        };
        let rest = line.strings.get(pos + 1..).unwrap_or_default();
        let sections = std::iter::once(first).chain(rest.iter().cloned());
        Cow::Owned(sections.collect())
    };

    Some(Diagnostic {
        level,
        path,
        line: loc_line,
        column: loc_col,
        message,
    })
}
