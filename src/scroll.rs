use {
    lazy_regex::*,
    std::fmt,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollCommand {
    Top,
    Bottom,
    Lines(i32),
    Pages(i32),
}

impl fmt::Display for ScrollCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn txt(n: i32, thing: &str, way: &str, f: &mut fmt::Formatter) -> fmt::Result {
            let p = if n > 1 { "s" } else { "" };
            write!(f, "scroll {n} {thing}{p} {way}")
        }
        match self {
            Self::Top => write!(f, "scroll to top"),
            Self::Bottom => write!(f, "scroll to bottom"),
            Self::Lines(lines) => if *lines > 0 {
                txt(*lines, "line", "down", f)
            } else {
                txt(-lines, "line", "up", f)
            }
            Self::Pages(pages) => if *pages > 0 {
                txt(*pages, "page", "down", f)
            } else {
                txt(-pages, "page", "up", f)
            }
        }
    }
}

impl ScrollCommand {
    fn to_lines(self, content_height: usize, page_height: usize) -> i32 {
        match self {
            Self::Top => -(content_height as i32),
            Self::Bottom => content_height as i32,
            Self::Lines(n) => n,
            Self::Pages(n) => n * page_height as i32,
        }
    }
    /// compute the new scroll value
    pub fn apply(self, scroll: usize, content_height: usize, page_height: usize) -> usize {
        if content_height > page_height {
            (scroll as i32 + self.to_lines(content_height, page_height))
                .min((content_height - page_height) as i32)
                .max(0) as usize
        } else {
            0
        }
    }
}

impl std::str::FromStr for ScrollCommand {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        if s == "scroll-to-top" {
            Ok(Self::Top)
        } else if s == "scroll-to-bottom" {
            Ok(Self::Bottom)
        } else if let Some((_, lines)) = regex_captures!(r#"^scroll-lines?\(([+-]?\d{1,4})\)$"#, s) {
            let lines = lines.parse().unwrap(); // can't fail
            Ok(Self::Lines(lines))
        } else if let Some((_, pages)) = regex_captures!(r#"^scroll-pages?\(([+-]?\d{1,4})\)$"#, s) {
            let pages = pages.parse().unwrap(); // can't fail
            Ok(Self::Pages(pages))
        } else {
            Err(())
        }
    }
}

pub fn is_thumb(y: usize, scrollbar: Option<(u16, u16)>) -> bool {
    scrollbar.map_or(false, |(sctop, scbottom)| {
        let y = y as u16;
        sctop <= y && y <= scbottom
    })
}

pub fn fix_scroll(scroll: usize, content_height: usize, page_height: usize) -> usize {
    if content_height > page_height {
        scroll.min(content_height - page_height - 1)
    } else {
        0
    }
}
