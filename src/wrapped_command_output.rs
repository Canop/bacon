use crate::*;

/// A wrapped cmd_output, only valid for the cmd_output it was computed for,
/// contains references to the start and end of lines wrapped for a
/// given width
pub struct WrappedCommandOutput {
    pub sub_lines: Vec<SubLine>,

    /// in order to allow partial wrapping, and assuming the wrapped part
    /// didn't change, we store the count of lines which were wrapped so
    /// that we may update starting from there
    pub wrapped_lines_count: usize,
}

impl WrappedCommandOutput {
    /// compute a new wrapped cmd_output for a width and cmd_output.
    ///
    /// width is the total area width, including the scrollbar.
    pub fn new(cmd_output: &CommandOutput, width: u16) -> Self {
        debug!("wrapping cmd_output");
        let sub_lines = wrap(&cmd_output.lines, width);
        Self {
            sub_lines,
            wrapped_lines_count: cmd_output.len(),
        }
    }

    /// Assuming the width is the same and the lines already handled
    /// didn't change, wrap and add the lines which weren't.
    pub fn update(&mut self, cmd_output: &CommandOutput, width: u16) {
        debug!("updating cmd_output");
        let mut new_lines = wrap(&cmd_output.lines[self.wrapped_lines_count..], width);
        for mut line in new_lines.drain(..) {
            line.line_idx += self.wrapped_lines_count;
            self.sub_lines.push(line);
        }
        self.wrapped_lines_count = cmd_output.lines.len();
    }
}
