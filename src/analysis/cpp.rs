use {
    crate::*,
    anyhow::Result,
    std::{
        borrow::Cow,
        iter::once,
    },
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
            items.push_line(LineType::Normal, line.content.clone());
        }
    }
    Ok(items.report())
}

/// Expect that diagnostics look like the following:
///
/// ```text
/// [path/to/source:line:column:] [level:] [message...]
/// ```
///
/// ... where each part is contained within a potentially styled section.
fn recognize_diagnostic(line: &TLine) -> Option<Diagnostic<'_>> {
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
        let sections = once(first).chain(rest.iter().cloned());
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

#[derive(Debug, Default)]
pub struct CppDoctestAnalyzer {
    lines: Vec<CommandOutputLine>,
}

impl Analyzer for CppDoctestAnalyzer {
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
        build_doctest_report(&self.lines)
    }
}

fn build_doctest_report(lines: &[CommandOutputLine]) -> Result<Report> {
    let mut items = ItemAccumulator::default();
    let mut current_test_case = String::from("(unknown test)");
    let mut empty_count = 0;

    for line in lines {
        match recognize_doctest_line(&current_test_case, &line.content) {
            Some(DoctestDiagnostic::StartTestCase(tc)) => current_test_case = tc.into(),
            Some(DoctestDiagnostic::Failure {
                level,
                location,
                first_line,
            }) => {
                let content = match level {
                    Kind::Error => burp::error_line_ts(&first_line),
                    Kind::Warning => burp::warning_line_ts(&first_line),
                    _ => unreachable!(),
                };

                items.push_failure_title(content);
                items.push_line(LineType::Location, burp::location_line(location));
                empty_count = 0;
            }
            Some(DoctestDiagnostic::StartOrEnd) => current_test_case = "(unknown test)".into(),
            None => {
                if empty_count < 1 {
                    items.push_line(LineType::Normal, line.content.clone());
                }
                if line.content.strings.iter().all(|l| l.raw.is_empty()) {
                    empty_count += 1;
                }
            }
        }
    }
    Ok(items.report())
}

enum DoctestDiagnostic<'a> {
    StartTestCase(&'a str),
    Failure {
        location: &'a str,
        level: Kind,
        first_line: Vec<TString>,
    },
    StartOrEnd,
}

fn recognize_doctest_line<'l>(
    current_test_case: &str,
    line: &'l TLine,
) -> Option<DoctestDiagnostic<'l>> {
    let get = |idx: usize| line.strings.get(idx).map(|l| l.raw.as_str());

    if let Some("===============================================================================") =
        get(0)
    {
        return Some(DoctestDiagnostic::StartOrEnd);
    }

    if let Some("TEST CASE:  ") = get(0) {
        return Some(DoctestDiagnostic::StartTestCase(
            line.strings.get(1).map_or("", |l| &l.raw),
        ));
    }

    if let Some(s @ ("WARNING: " | "ERROR: " | "FATAL ERROR: ")) = get(1) {
        let level = match s {
            "WARNING: " => Kind::Warning,
            "ERROR: " | "FATAL ERROR: " => Kind::Error,
            _ => unreachable!(),
        };

        let rest_of_line = once(TString::new("", current_test_case))
            .chain(once(TString::new("", " ")))
            .chain(line.strings.get(2..).unwrap_or_default().iter().cloned())
            .collect();
        return Some(DoctestDiagnostic::Failure {
            level,
            location: line.strings.first().unwrap().raw.as_str().trim(),
            first_line: rest_of_line,
        });
    }

    None
}
