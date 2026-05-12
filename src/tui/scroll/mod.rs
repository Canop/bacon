mod scroll_anchor;
mod scroll_command;

pub use {
    scroll_anchor::*,
    scroll_command::ScrollCommand,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollEnd {
    First,
    Last,
}

pub enum VisibleScrollState {
    ScrollEnd(ScrollEnd),
    TopItemIdx(usize),
}

impl ScrollEnd {
    pub fn reverse(self) -> Self {
        match self {
            ScrollEnd::First => ScrollEnd::Last,
            ScrollEnd::Last => ScrollEnd::First,
        }
    }
}

pub fn is_thumb(
    y: usize,
    scrollbar: Option<(u16, u16)>,
) -> bool {
    scrollbar.is_some_and(|(sctop, scbottom)| {
        let y = y as u16;
        sctop <= y && y <= scbottom
    })
}

pub fn fix_scroll(
    scroll: usize,
    content_height: usize,
    page_height: usize,
) -> usize {
    if content_height > page_height {
        scroll.min(content_height - page_height)
    } else {
        0
    }
}
