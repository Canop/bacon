use {
    crate::*,
    anyhow::*,
    std::io::Write,
    unicode_width::UnicodeWidthChar,
};

#[derive(Debug, Default)]
pub struct SubString {
    pub string_idx: usize,
    pub byte_idx: usize,
}
impl SubString {
    pub fn draw(&self, w: &mut W, report: &Report, line_idx: usize) -> Result<()> {
        let line = &report.lines[line_idx];
        let string = &line.content.strings[self.string_idx];
        if string.csi.is_empty() {
            write!(w, "{}", &string.raw[self.byte_idx..])?;
        } else {
            write!(
                w,
                "{}{}{}",
                &string.csi,
                &string.raw[self.byte_idx..],
                CSI_RESET,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SubLine {
    line_idx: usize,
    sub_strings: Vec<SubString>,
}

impl SubLine {
    pub fn is_continuation(&self) -> bool {
        self.sub_strings.get(0)
            .map_or(false, |sub_string| {
                sub_string.string_idx != 0 || sub_string.byte_idx != 0
            })
    }
    pub fn src_line<'r>(&self, report: &'r Report) -> &'r Line {
        &report.lines[self.line_idx]
    }
    pub fn src_line_type(&self, report: &Report) -> LineType {
        report.lines[self.line_idx].line_type
    }
    pub fn line_type(&self, report: &Report) -> LineType {
        if self.is_continuation() {
            LineType::Normal
        } else {
            report.lines[self.line_idx].line_type
        }
    }
    pub fn draw_line_type(&self, w: &mut W, report: &Report) -> Result<()> {
        self.line_type(report).draw(w, report.lines[self.line_idx].item_idx)?;
        Ok(())
    }
    pub fn draw(&self, w: &mut W, report: &Report) -> Result<()> {
        for ts in &self.sub_strings {
            ts.draw(w, report, self.line_idx)?;
        }
        Ok(())
    }
}

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
        let cols = width as usize - 1; // -1 for the probable scrollbar
        let lines = &report.lines;
        let mut sub_lines = Vec::new();
        for line_idx in 0..lines.len() {
            sub_lines.push(SubLine { line_idx, sub_strings: Vec::new() });
            let mut sub_cols = lines[line_idx].line_type.cols();
            let strings = &lines[line_idx].content.strings;
            for string_idx in 0..strings.len() {
                let string = &strings[string_idx];
                sub_lines.last_mut().unwrap().sub_strings.push(SubString { string_idx, byte_idx:0 });
                for (byte_idx, c) in string.raw.char_indices() {
                    let char_cols = c.width().unwrap_or(0);
                    if sub_cols + char_cols > cols && sub_cols > 0 {
                        sub_lines.push(SubLine {
                            line_idx,
                            sub_strings: vec![ SubString { string_idx, byte_idx } ],
                        });
                        sub_cols = char_cols;
                    } else {
                        sub_cols += char_cols;
                    }
                }
            }
        }
        Self { sub_lines }
    }
}
