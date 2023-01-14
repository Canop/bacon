use {
    crate::*,
    anyhow::*,
    std::io::Write,
    unicode_width::UnicodeWidthChar,
};

/// a line which can be wrapped
pub trait WrappableLine {
    fn content(&self) -> &TLine;
    fn prefix_cols(&self) -> usize;
}

#[derive(Debug, Default)]
pub struct SubString {
    pub string_idx: usize,
    pub byte_start: usize,
    pub byte_end: usize, // not included
}
impl SubString {
    pub fn draw(
        &self,
        w: &mut W,
        content: &TLine,
    ) -> Result<()> {
        //let line = &report.lines[line_idx];
        let string = &content.strings[self.string_idx];
        if string.csi.is_empty() {
            write!(w, "{}", &string.raw[self.byte_start..self.byte_end])?;
        } else {
            write!(
                w,
                "{}{}{}",
                &string.csi,
                &string.raw[self.byte_start..self.byte_end],
                CSI_RESET,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct SubLine {
    pub line_idx: usize,
    pub sub_strings: Vec<SubString>,
}

impl SubLine {
    pub fn is_continuation(&self) -> bool {
        self.sub_strings.get(0).map_or(false, |sub_string| {
            sub_string.string_idx != 0 || sub_string.byte_start != 0
        })
    }
    pub fn src_line<'r>(
        &self,
        report: &'r Report,
    ) -> &'r Line {
        &report.lines[self.line_idx]
    }
    pub fn src_line_type(
        &self,
        report: &Report,
    ) -> LineType {
        report.lines[self.line_idx].line_type
    }
    pub fn line_type(
        &self,
        report: &Report,
    ) -> LineType {
        if self.is_continuation() {
            LineType::Normal
        } else {
            report.lines[self.line_idx].line_type
        }
    }
    pub fn draw_line_type(
        &self,
        w: &mut W,
        report: &Report,
    ) -> Result<()> {
        self.line_type(report)
            .draw(w, report.lines[self.line_idx].item_idx)?;
        Ok(())
    }
    pub fn draw<WL: WrappableLine>(
        &self,
        w: &mut W,
        source_lines: &[WL],
    ) -> Result<()> {
        for ts in &self.sub_strings {
            let content = &source_lines[self.line_idx].content();
            ts.draw(w, content)?;
        }
        Ok(())
    }
}

pub fn wrap<WL: WrappableLine>(
    lines: &[WL],
    width: u16,
) -> Vec<SubLine> {
    let cols = width as usize - 1; // -1 for the probable scrollbar
    let mut sub_lines = Vec::new();
    for (line_idx, line) in lines.iter().enumerate() {
        sub_lines.push(SubLine {
            line_idx,
            sub_strings: Vec::new(),
        });
        let mut sub_cols = line.prefix_cols();
        let strings = &line.content().strings;
        for (string_idx, string) in strings.iter().enumerate() {
            sub_lines.last_mut().unwrap().sub_strings.push(SubString {
                string_idx,
                byte_start: 0,
                byte_end: string.raw.len(), // may be changed later on cut
            });
            for (byte_idx, c) in string.raw.char_indices() {
                let char_cols = c.width().unwrap_or(0);
                if sub_cols + char_cols > cols && sub_cols > 0 {
                    sub_lines
                        .last_mut()
                        .unwrap()
                        .sub_strings
                        .last_mut()
                        .unwrap()
                        .byte_end = byte_idx;
                    sub_lines.push(SubLine {
                        line_idx,
                        sub_strings: vec![SubString {
                            string_idx,
                            byte_start: byte_idx,
                            byte_end: string.raw.len(), // may be changed later on cut
                        }],
                    });
                    sub_cols = char_cols;
                } else {
                    sub_cols += char_cols;
                }
            }
        }
    }
    sub_lines
}
