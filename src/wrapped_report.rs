use {
    crate::*,
};

/// A wrapped report, only valid for the report it was computed for,
/// contains references to the start and end of lines wrapped for a
/// given width
pub struct WrappedReport {
    pub sub_lines: Vec<SubLine>,
}

impl WrappedReport {
    /// compute a new wrapped report for a width and report.
    ///
    /// width is the total area width, including the scrollbar.
    pub fn new(report: &Report, width: u16) -> Self {
        debug!("wrapping report");
        let sub_lines = wrap(&report.lines, width);
        Self { sub_lines }
    }
}
