//! An analyzer for biome ( https://biomejs.dev/ )

use {
    super::*,
    crate::*,
    anyhow::Result,
};

#[derive(Debug, Default)]
pub struct BiomeAnalyzer {
    lines: Vec<CommandOutputLine>,
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

/// Build a report from the output of eslint
///
/// The main specificity of eslint is that the path of a file with error is given
/// before the errors of the file, each error coming with the line and column
/// in the file.
pub fn build_report(cmd_lines: &[CommandOutputLine]) -> anyhow::Result<Report> {
    info!("biome::build_report =========");
    let mut items = ItemAccumulator::default();
    let mut i = 0;
    for cmd_line in cmd_lines {
        if i < 4 {
            info!("line {}: {:#?}", i, cmd_line);
            i+=1;
        }
    }
    let lines = items.lines();
    let stats = Stats::from(&lines);
    info!("stats: {:#?}", &stats);
    let report = Report {
        lines,
        stats,
        suggest_backtrace: false,
        output: Default::default(),
        failure_keys: Vec::new(),
    };
    Ok(report)
}

