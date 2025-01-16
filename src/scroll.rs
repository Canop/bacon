use {
    lazy_regex::*,
    serde::{
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
        de,
    },
    std::{
        fmt,
        str::FromStr,
    },
};

/// A scroll related command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollCommand {
    Top,
    Bottom,
    Lines(i32),
    MilliPages(i32),
}

impl ScrollCommand {
    pub fn pages(n: i32) -> Self {
        Self::MilliPages(n * 1000)
    }
    fn to_lines(
        self,
        content_height: usize,
        page_height: usize,
    ) -> i32 {
        match self {
            Self::Top => -(content_height as i32),
            Self::Bottom => content_height as i32,
            Self::Lines(n) => n,
            Self::MilliPages(n) => {
                let lines = n as f64 * page_height as f64 / 1000.0;
                let lines = if lines < 0.0 {
                    lines.floor()
                } else {
                    lines.ceil()
                };
                lines as i32
            }
        }
    }
    /// Return the action description to show in doc/help
    pub fn doc(&self) -> String {
        fn txt(
            n: i32,
            thing: &str,
            way: &str,
        ) -> String {
            let p = if n > 1 { "s" } else { "" };
            format!("scroll {n} {thing}{p} {way}")
        }
        match self {
            Self::Top => "scroll to top".to_string(),
            Self::Bottom => "scroll to bottom".to_string(),
            Self::Lines(lines) => {
                if *lines > 0 {
                    txt(*lines, "line", "down")
                } else {
                    txt(-lines, "line", "up")
                }
            }
            Self::MilliPages(n) => {
                if n % 1000 == 0 {
                    let pages = n / 1000;
                    if pages > 0 {
                        txt(pages, "page", "down")
                    } else {
                        txt(-pages, "page", "up")
                    }
                } else {
                    let pages: f64 = *n as f64 / 1000.0;
                    let s = format!("{pages:.3}");
                    let s = s.trim_matches('0');
                    format!("scroll {s} pages")
                }
            }
        }
    }
    /// compute the new scroll value
    pub fn apply(
        self,
        scroll: usize,
        content_height: usize,
        page_height: usize,
    ) -> usize {
        if content_height > page_height {
            (scroll as i32 + self.to_lines(content_height, page_height))
                .min((content_height - page_height) as i32)
                .max(0) as usize
        } else {
            0
        }
    }
}

impl fmt::Display for ScrollCommand {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::Top => write!(f, "scroll-to-top"),
            Self::Bottom => write!(f, "scroll-to-bottom"),
            Self::Lines(n) => write!(f, "scroll-lines({n})"),
            Self::MilliPages(n) => {
                if n % 1000 == 0 {
                    let n = n / 1000;
                    write!(f, "scroll-pages({n})")
                } else {
                    let n: f64 = *n as f64 / 1000.0;
                    let s = format!("{n:.3}");
                    let s = s.trim_matches('0');
                    write!(f, "scroll-pages({s})")
                }
            }
        }
    }
}
impl std::str::FromStr for ScrollCommand {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        regex_switch!(s,
            "^scroll[-_]?to[-_]?top$"i => Self::Top,
            "^scroll[-_]?to[-_]?bottom$"i => Self::Bottom,
            r#"^scroll[-_]?lines?\((?<n>[+-]?\d{1,4})\)$"#i => Self::Lines(
                n.parse().unwrap() // can't fail because [+-]?\d{1,4}
            ),
            r#"^scroll[-_]?pages?\((?<n>[+-]?\d{1,4})\)$"#i => Self::MilliPages(
                n.parse::<i32>().unwrap() * 1000 // can't fail because [+-]?\d{1,4}
            ),
            r#"^scroll[-_]?pages?\((?<n>[+-]?\d*\.\d{1,3})\)$"#i => {
                let n: f64 = n.parse().unwrap(); // can't fail
                let n: i32 = (n * 1000.0).round() as i32;
                Self::MilliPages(n)
            }
        )
        .ok_or("not a valid scroll command")
    }
}

impl Serialize for ScrollCommand {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}
impl<'de> Deserialize<'de> for ScrollCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
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
        scroll.min(content_height - page_height - 1)
    } else {
        0
    }
}

#[test]
fn test_scroll_command_string_round_trip() {
    let commands = [
        ScrollCommand::Lines(3),
        ScrollCommand::Lines(-12),
        ScrollCommand::MilliPages(1000),
        ScrollCommand::MilliPages(-2000),
        ScrollCommand::Top,
        ScrollCommand::Bottom,
    ];
    for command in commands {
        assert_eq!(command.to_string().parse(), Ok(command));
    }
}
#[test]
fn test_scroll_command_string_alternative_writings() {
    assert_eq!("SCROLL-TO-TOP".parse(), Ok(ScrollCommand::Top));
    assert_eq!("ScrollLines(5)".parse(), Ok(ScrollCommand::Lines(5)));
    assert_eq!("scroll-lines(+12)".parse(), Ok(ScrollCommand::Lines(12)));
    assert_eq!(
        "scroll_pages(-2)".parse(),
        Ok(ScrollCommand::MilliPages(-2000))
    );
    assert_eq!(
        "scroll_pages(-.2)".parse(),
        Ok(ScrollCommand::MilliPages(-200))
    );
}
