use {
    crate::*,
    unicode_width::UnicodeWidthChar,
};

/// Wrap lines into sublines containing positions in the original lines
pub fn wrap(
    lines: &[Line],
    width: u16,
) -> Vec<Line> {
    let cols = width as usize - 1; // -1 for the probable scrollbar
    let mut sub_lines = Vec::new();
    for line in lines.iter() {
        let summary = line.line_type.is_summary();
        sub_lines.push(Line {
            item_idx: line.item_idx,
            content: TLine::default(),
            line_type: line.line_type,
        });
        let mut sub_cols = line.line_type.cols();
        let mut wrap_idx = 0; // 1 for first continuation, etc.
        for string in &line.content.strings {
            sub_lines
                .last_mut()
                .unwrap()
                .content
                .strings
                .push(string.clone()); // might be truncated later
            let mut byte_offset = 0;
            for (byte_idx, c) in string.raw.char_indices() {
                let char_cols = c.width().unwrap_or(0);
                if sub_cols + char_cols > cols && sub_cols > 0 {
                    let last_string = sub_lines
                        .last_mut()
                        .unwrap()
                        .content
                        .strings
                        .last_mut()
                        .unwrap();
                    let after_cut = TString::new(
                        last_string.csi.clone(),
                        last_string.raw[byte_idx - byte_offset..].to_string(),
                    );
                    last_string.raw.truncate(byte_idx - byte_offset);
                    byte_offset = byte_idx;
                    sub_lines.push(Line {
                        item_idx: line.item_idx,
                        content: TLine {
                            strings: vec![after_cut],
                        },
                        line_type: LineType::Continuation {
                            offset: wrap_idx,
                            summary,
                        },
                    });
                    wrap_idx += 1;
                    sub_cols = char_cols;
                } else {
                    sub_cols += char_cols;
                }
            }
        }
    }
    sub_lines
}
