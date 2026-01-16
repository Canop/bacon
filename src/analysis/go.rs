//! An analyzer for the Go compiler `go`

use {super::*, crate::*, anyhow::Result, lazy_regex::*};

#[derive(Debug, Default)]
pub struct GoAnalyzer {
    lines: Vec<CommandOutputLine>,
}

#[derive(Debug)]
struct LocationCode {
    location: String,
    code: String,
}

#[derive(Debug)]
enum GoLine {
    LocationCode(LocationCode),
    Other,
}

impl Analyzer for GoAnalyzer {
    fn start(
        &mut self,
        _: &Mission,
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

fn recognize_line(tline: &TLine) -> GoLine {
    if let Some(lc) = recognize_location_code(tline) {
        return GoLine::LocationCode(lc);
    }
    GoLine::Other
}

fn recognize_location_code(tline: &TLine) -> Option<LocationCode> {
    let raw = tline.to_raw();
    if let Some((_, location, code)) = regex_captures!(r"([^\s:]+:\d+:\d+): (.+$)", &raw) {
        return Some(LocationCode {
            location: location.to_string(),
            code: code.to_string(),
        });
    }
    None
}

/// Build a report from the output of biome
pub fn build_report(cmd_lines: &[CommandOutputLine]) -> anyhow::Result<Report> {
    let mut items = ItemAccumulator::default();
    let mut last_is_blank = true;
    for cmd_line in cmd_lines {
        let bline = recognize_line(&cmd_line.content);
        if let GoLine::LocationCode(lc) = bline {
            let error_line = burp::error_line(&lc.code);
            items.push_error_title(error_line);
            items.push_line(
                LineType::Location,
                burp::location_line(lc.location.to_string()),
            );
            last_is_blank = false;
        } else {
            let is_blank = cmd_line.content.is_blank();
            if !(is_blank && last_is_blank) {
                items.push_line(LineType::Normal, cmd_line.content.clone());
            }
            last_is_blank = is_blank;
        }
    }
    Ok(items.report())
}
