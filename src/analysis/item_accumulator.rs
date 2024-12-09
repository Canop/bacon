use crate::*;

/// Receives lines and accumulates them into items, used
/// at end to build a sorted list of lines.
///
/// This is a small optional utility for report makers
#[derive(Default)]
pub struct ItemAccumulator {
    curr_kind: Option<Kind>,
    errors: Vec<Line>,
    test_fails: Vec<Line>,
    warnings: Vec<Line>,
}

impl ItemAccumulator {
    pub fn start_item(
        &mut self,
        kind: Kind,
    ) {
        self.curr_kind = Some(kind);
    }
    pub fn close_item(&mut self) {
        self.curr_kind = None;
    }
    pub fn push_line(
        &mut self,
        line_type: LineType,
        content: TLine,
    ) {
        let line = Line {
            item_idx: 0, // will be filled later
            line_type,
            content,
        };
        match self.curr_kind {
            Some(Kind::Warning) => self.warnings.push(line),
            Some(Kind::Error) => self.errors.push(line),
            Some(Kind::TestFail) => self.test_fails.push(line),
            _ => {} // before warnings and errors, or in a sum
        }
    }
    pub fn push_error_title(
        &mut self,
        content: TLine,
    ) {
        self.curr_kind = Some(Kind::Error);
        self.push_line(LineType::Title(Kind::Error), content);
    }
    pub fn push_failure_title(
        &mut self,
        content: TLine,
    ) {
        self.curr_kind = Some(Kind::TestFail);
        self.push_line(LineType::Title(Kind::TestFail), content);
    }
    pub fn lines(mut self) -> Vec<Line> {
        let mut lines = self.errors;
        lines.append(&mut self.test_fails);
        lines.append(&mut self.warnings);
        let mut item_idx = 0;
        for line in &mut lines {
            if matches!(line.line_type, LineType::Title(_)) {
                item_idx += 1;
            }
            line.item_idx = item_idx;
        }
        lines
    }
    pub fn report(self) -> Report {
        let lines = self.lines();
        let stats = Stats::from(&lines);
        info!("stats: {:#?}", &stats);
        Report {
            lines,
            stats,
            suggest_backtrace: false,
            output: Default::default(),
            failure_keys: Vec::new(),
            analyzer_exports: Default::default(),
        }
    }
}
