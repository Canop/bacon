use crate::*;

/// A wrapped report, only valid for the report it was computed for,
/// contains references to the start and end of lines wrapped for a
/// given width
pub struct WrappedReport {
    pub sub_lines: Vec<SubLine>,
    pub summary_height: usize,
}

impl WrappedReport {
    /// compute a new wrapped report for a width and report.
    ///
    /// width is the total area width, including the scrollbar.
    pub fn new(report: &Report, width: u16) -> Self {
        debug!("wrapping report");
        let sub_lines = wrap(&report.lines, width);
        let summary_height = sub_lines
            .iter()
            .filter(|sl| {
                report
                    .lines
                    .get(sl.line_idx)
                    .map_or(true, |l| l.line_type != LineType::Normal)
            })
            .count();
        Self {
            sub_lines,
            summary_height,
        }
    }
    pub fn content_height(&self, summary: bool) -> usize {
        if summary {
            self.summary_height
        } else {
            self.sub_lines.len()
        }
    }
}
