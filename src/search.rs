use crate::*;

// temporary struct, will need several tranges
#[derive(Debug, PartialEq, Eq)]
pub struct Found {
    pub line_idx: usize,
    pub trange: TRange,
}

pub struct Pattern {
    pub pattern: String, // might change later
}

pub const CSI_FOUND: &str = "\u{1b}[1m\u{1b}[38;5;208m"; // bold, orange foreground
pub const CSI_FOUND_SELECTED: &str = "\u{1b}[1m\u{1b}[30m\u{1b}[48;5;208m"; // bold, orange background

impl Pattern {
    // Current limitations:
    // - match can't be split between two tstrings (and thus is broken on wrap)
    pub fn search_lines(
        &self,
        lines: &[Line],
    ) -> Vec<Found> {
        let pattern = &self.pattern;
        let mut founds = Vec::new();
        for (line_idx, line) in lines.iter().enumerate() {
            for (string_idx, tstring) in line.content.strings.iter().enumerate() {
                let mut offset = 0;
                while offset + pattern.len() < tstring.raw.len() {
                    let haystack = &tstring.raw[offset..];
                    let Some(pos) = haystack.find(pattern) else {
                        break;
                    };
                    let found = Found {
                        line_idx,
                        trange: TRange {
                            string_idx,
                            start_byte_in_string: pos + offset,
                            end_byte_in_string: pos + offset + pattern.len(),
                        },
                    };
                    founds.push(found);
                    offset += pos + pattern.len();
                }
            }
        }
        founds
    }
}
