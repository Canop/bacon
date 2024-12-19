use crate::*;

pub struct WrappedReport {
    pub sub_lines: Vec<Line>,
    /// number of summary lines after wrapping
    pub summary_height: usize,
}

impl WrappedReport {
    /// compute a new wrapped report for a width and report.
    ///
    /// width is the total area width, including the scrollbar.
    pub fn new(
        report: &Report,
        width: u16,
    ) -> Self {
        debug!("wrapping report");
        let sub_lines = wrap(&report.lines, width);
        let summary_height = sub_lines
            .iter()
            .filter(|sl| sl.line_type.is_summary())
            .count();
        Self {
            sub_lines,
            summary_height,
        }
    }
}
