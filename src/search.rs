use crate::*;

/// position in a [TLine] of a found pattern
#[derive(Debug, PartialEq, Eq)]
pub struct Found {
    /// The index of the first line containing the pattern
    pub line_idx: usize,
    /// The range of the pattern in the line
    pub trange: TRange,
    /// If the pattern goes over a line wrap, the range of the pattern in the next line
    pub continued: Option<TRange>,
}

pub struct Pattern {
    pub pattern: String, // might change later
}

pub const CSI_FOUND: &str = "\u{1b}[1m\u{1b}[38;5;208m"; // bold, orange foreground
pub const CSI_FOUND_SELECTED: &str = "\u{1b}[1m\u{1b}[30m\u{1b}[48;5;208m"; // bold, orange background

impl Pattern {
    // Current limitations:
    // - a match can't span over more than 2 lines. This is probably fine.
    pub fn search_lines<'i, I>(
        &self,
        lines: I,
    ) -> Vec<Found>
    where
        I: IntoIterator<Item = &'i Line>,
    {
        let lines = lines.into_iter();
        let pattern = &self.pattern;
        let len = pattern.len();
        let mut founds = Vec::new();
        let mut previous_line: Option<&Line> = None;
        for (line_idx, line) in lines.enumerate() {
            if line.is_continuation() {
                if let Some(previous_line) = previous_line {
                    // we check for a match broken by wrapping
                    if !previous_line.content.strings.is_empty() && !line.content.strings.is_empty()
                    {
                        let previous_line_string_idx = previous_line.content.strings.len() - 1;
                        let previous_last_raw =
                            &previous_line.content.strings[previous_line_string_idx].raw;
                        if let Some(cut) = find_cut_pattern(
                            pattern,
                            previous_last_raw,
                            &line.content.strings[0].raw,
                        ) {
                            let found = Found {
                                line_idx: line_idx - 1,
                                trange: TRange {
                                    string_idx: previous_line_string_idx,
                                    start_byte_in_string: previous_last_raw.len() - cut,
                                    end_byte_in_string: previous_last_raw.len(),
                                },
                                continued: Some(TRange {
                                    string_idx: 0,
                                    start_byte_in_string: 0,
                                    end_byte_in_string: len - cut,
                                }),
                            };
                            founds.push(found);
                        }
                    }
                }
            }
            previous_line = Some(line);
            for (string_idx, tstring) in line.content.strings.iter().enumerate() {
                let mut offset = 0;
                while offset + len < tstring.raw.len() {
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
                        continued: None,
                    };
                    founds.push(found);
                    offset += pos + pattern.len();
                }
            }
        }
        founds
    }
}

fn find_cut_pattern(
    pattern: &str,
    a: &str,
    b: &str,
) -> Option<usize> {
    let len = pattern.len();
    (1..len).find(|&i| a.ends_with(&pattern[..i]) && b.starts_with(&pattern[i..]))
}
