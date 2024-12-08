//! An analyzer for biome ( https://biomejs.dev/ )

use {
    super::*,
    crate::*,
    anyhow::Result,
    lazy_regex::*,
};

#[derive(Debug, Default)]
pub struct BiomeAnalyzer {
    lines: Vec<CommandOutputLine>,
}

#[derive(Debug)]
struct LocationCode<'l> {
    location: &'l str,        // path with line and column
    code: &'l str,            // eg "lint/complexity/noForEach"
    tag: Option<&'l TString>, // eg "FIXABLE"
}

#[derive(Debug)]
enum BiomeLine<'l> {
    LocationCode(LocationCode<'l>),
    Other,
}

impl Analyzer for BiomeAnalyzer {
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

fn recognize_line(tline: &TLine) -> BiomeLine {
    if let Some(lc) = recognize_location_code(tline) {
        return BiomeLine::LocationCode(lc);
    }
    BiomeLine::Other
}

fn recognize_location_code(tline: &TLine) -> Option<LocationCode> {
    if let Some(s) = tline.if_unstyled() {
        // untagged
        if let Some((_, location, code)) = regex_captures!(r"([^\s:]+:\d+:\d+) (\S+) ━+$", s) {
            let tag = None;
            return Some(LocationCode {
                location,
                code,
                tag,
            });
        }
    }
    let mut strings = tline.strings.iter();
    let a = strings.next();
    let b = strings.next();
    let c = strings.next();
    if let (Some(a), Some(b), Some(c)) = (a, b, c) {
        if a.is_unstyled() && c.is_unstyled() && regex_is_match!("^ ━+$", &c.raw) {
            if let Some((_, location, code)) = regex_captures!(r"([^\s:]+:\d+:\d+) (\S+) ", &a.raw)
            {
                let tag = Some(b);
                return Some(LocationCode {
                    location,
                    code,
                    tag,
                });
            }
        }
    }
    None
}

/// Build a report from the output of biome
pub fn build_report(cmd_lines: &[CommandOutputLine]) -> anyhow::Result<Report> {
    let mut items = ItemAccumulator::default();
    let mut last_is_blank = true;
    for cmd_line in cmd_lines {
        let bline = recognize_line(&cmd_line.content);
        if let BiomeLine::LocationCode(lc) = bline {
            let mut error_line = burp::error_line(lc.code);
            if let Some(tag) = lc.tag {
                error_line.strings.push(TString::new("", " "));
                error_line.strings.push(tag.clone());
            }
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
