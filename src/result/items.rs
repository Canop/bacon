use crate::*;

#[derive(Clone, Copy)]
pub struct Item<'l> {
    /// non empty slice of lines with the same item_idx
    lines: &'l [Line],
}

impl<'l> Item<'l> {
    /// return a vector of slices of lines, each slice pointing to the
    /// consecutive lines having the same item_idx
    pub fn items_of(lines: &'l [Line]) -> Vec<Self> {
        let mut items: Vec<Self> = Vec::new();
        let mut start = 0;
        for i in 1..lines.len() {
            if lines[i].item_idx != lines[start].item_idx {
                items.push(Item {
                    lines: &lines[start..i],
                });
                start = i;
            }
        }
        if start < lines.len() {
            items.push(Item {
                lines: &lines[start..lines.len()],
            });
        }
        items
    }

    pub fn item_idx(&self) -> usize {
        self.lines[0].item_idx
    }

    pub fn lines(&self) -> &'l [Line] {
        self.lines
    }

    pub fn location(&self) -> Option<&str> {
        for line in self.lines {
            if let Some(location) = line.location() {
                return Some(location);
            }
        }
        None
    }
    pub fn diag_type(&self) -> Option<&str> {
        for line in self.lines {
            if let Some(diag_type) = line.diag_type() {
                return Some(diag_type);
            }
        }
        None
    }
}
