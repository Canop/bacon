//! An analyzer for ruff ( https://docs.astral.sh/ruff/ )

use {
    crate::*,
    anyhow::Result,
    lazy_regex::*,
};

#[derive(Debug, Default)]
pub struct RuffAnalyzer {
    lines: Vec<CommandOutputLine>,
}

#[derive(Debug)]
struct LocationTitle<'l> {
    path: &'l str,
    line: &'l str,
    column: &'l str,
    message: &'l [TString],
}

#[derive(Debug)]
enum RuffLine<'l> {
    LocationTitle(LocationTitle<'l>),
    Other,
}

impl Analyzer for RuffAnalyzer {
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

fn recognize_line(tline: &TLine) -> RuffLine {
    if let Some(lt) = recognize_location_message(tline) {
        return RuffLine::LocationTitle(lt);
    }
    RuffLine::Other
}

fn recognize_location_message(tline: &TLine) -> Option<LocationTitle> {
    let tstrings = &tline.strings;
    if tstrings.len() < 8 {
        return None;
    }
    if tstrings[0].csi != "\u{1b}[1m" || tstrings[1].raw.is_empty() // path
        || tstrings[1].csi != "\u{1b}[36m" || tstrings[1].raw != ":"
        || tstrings[2].is_styled() || !regex_is_match!(r"^\d+$", &tstrings[2].raw) // line
        || tstrings[3].csi != "\u{1b}[36m" || tstrings[3].raw != ":"
        || tstrings[4].is_styled() || !regex_is_match!(r"^\d+$", &tstrings[4].raw) // column
        || tstrings[5].csi != "\u{1b}[36m" || tstrings[5].raw != ":"
        || tstrings[6].is_styled() || tstrings[6].raw != " "
    {
        return None;
    }
    Some(LocationTitle {
        path: &tstrings[0].raw,
        line: &tstrings[2].raw,
        column: &tstrings[4].raw,
        message: &tstrings[7..],
    })
}

/// Build a report from the output of biome
pub fn build_report(cmd_lines: &[CommandOutputLine]) -> anyhow::Result<Report> {
    let mut items = ItemAccumulator::default();
    let mut last_is_blank = true;
    let mut i = 0;
    for cmd_line in cmd_lines {
        if i < 5 {
            info!("cmd_line: {:#?}", &cmd_line);
            i += 1;
        }
        let bline = recognize_line(&cmd_line.content);
        if let RuffLine::LocationTitle(LocationTitle {
            path,
            line,
            column,
            message,
        }) = bline
        {
            items.push_error_title(burp::error_line_ts(message));
            items.push_line(
                LineType::Location,
                burp::location_line(format!("{}:{}:{}", path, line, column)),
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
