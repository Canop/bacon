use crate::*;

pub fn search_item_idx<'i, I>(
    idx: usize,
    lines: I,
) -> Vec<Found>
where
    I: IntoIterator<Item = &'i Line>,
{
    for (line_idx, line) in lines.into_iter().enumerate() {
        if line.item_idx == idx && !line.content.strings.is_empty() {
            let end_byte_in_string = line.content.strings[0].raw.len();
            return vec![Found {
                line_idx,
                trange: TRange {
                    string_idx: 0,
                    start_byte_in_string: 0,
                    end_byte_in_string,
                },
                continued: None,
            }];
        }
    }
    vec![]
}
