use {
    crate::*,
    anyhow::*,
    crossterm::{
        cursor, execute,
        style::{Color::*, Colorize, Print, Styler},
    },
    minimad::{Alignment, Composite},
    std::{io::Write, path::Path},
    termimad::{Area, CompoundStyle, MadSkin},
};


/// contains the currently rendered state of the application,
/// including scroll level and the current report (if any)
pub struct AppState {
    pub name: String,
    report: Option<Report>,
    width: u16,
    height: u16,
    pub computing: bool,
    pub summary: bool,
    status_skin: MadSkin,
    scroll: usize,
}
impl AppState {
    pub fn new(root_dir: &Path) -> Result<Self> {
        let mut status_skin = MadSkin::default();
        status_skin.paragraph.set_bg(DarkGrey);
        status_skin.italic = CompoundStyle::with_fg(Yellow);
        let (width, height) = termimad::terminal_size();
        Ok(Self {
            name: root_dir.file_name().unwrap().to_string_lossy().to_string(),
            report: None,
            width,
            height,
            computing: true,
            summary: false,
            status_skin,
            scroll: 0,
        })
    }
}

impl AppState {
    pub fn set_report(&mut self, report: Report) {
        self.scroll = 0;
        self.report = Some(report);
    }
    fn page_height(&self) -> usize {
        self.height.max(3) as usize - 3
    }
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }
    pub fn apply_scroll_command(&mut self, cmd: ScrollCommand) {
        if let Some(report) = &self.report {
            self.scroll = cmd.apply(
                self.scroll,
                report.stats.lines(self.summary),
                self.page_height(),
            );
        }
    }
    pub fn scroll(&mut self, w: &mut W, cmd: ScrollCommand) -> Result<()> {
        self.apply_scroll_command(cmd);
        self.draw(w)
    }
    fn draw_status(&self, w: &mut W, status: &str) -> Result<()> {
        if self.height > 1 {
            goto(w, self.height)?;
            self.status_skin.write_composite_fill(
                w,
                Composite::from_inline(status),
                self.width.into(),
                Alignment::Left,
            )?;
        }
        Ok(())
    }
    pub fn draw(&mut self, w: &mut W) -> Result<()> {
        let width = self.width as usize;
        goto(w, 1)?;
        if self.computing {
            eprint!(
                "{}",
                format!("{:^w$}", "computing...", w = width)
                    .white()
                    .on_dark_grey()
            );
        }
        if let Some(report) = &self.report {
            let stats = &report.stats;
            goto(w, 0)?;
            eprint!(
                "{} ",
                format!(" {} ", &self.name).white().bold().on_dark_grey()
            );
            if stats.errors > 0 {
                let s = if stats.errors == 1 {
                    " 1 error ".to_string()
                } else {
                    format!(" {} errors ", stats.errors)
                };
                eprint!("{} ", s.black().bold().on_red());
            }
            if stats.warnings > 0 {
                let s = if stats.warnings == 1 {
                    " 1 warning ".to_string()
                } else {
                    format!(" {} warnings ", stats.warnings)
                };
                eprint!("{} ", s.black().bold().on_yellow());
            }
            if stats.items() == 0 {
                eprint!("{} ", " pass! ".white().bold().on_dark_green());
            }
            if self.height < 4 {
                return self.draw_status(w, "terminal too small");
            }
            let mut area = Area::new(0, 2, self.width, self.page_height() as u16);
            let content_height = report.stats.lines(self.summary);
            let scrollbar = area.scrollbar(self.scroll as i32, content_height as i32);
            if scrollbar.is_some() {
                area.width -= 1;
            }
            let mut lines = report
                .lines
                .iter()
                .filter(|line| !(self.summary && line.line_type == LineType::Normal))
                .skip(self.scroll);
            for row_idx in 0..area.height {
                let y = row_idx + area.top;
                goto(w, y)?;
                if let Some(Line {
                    item_idx,
                    line_type,
                    content,
                }) = lines.next()
                {
                    match line_type {
                        LineType::Title(Kind::Error) => {
                            eprint!(
                                "{} {}",
                                format!("{:>2} ", item_idx).black().bold().on_red(),
                                &content,
                            );
                        }
                        LineType::Title(Kind::Warning) => {
                            eprint!(
                                "{} {}",
                                format!("{:>2} ", item_idx).black().bold().on_yellow(),
                                &content,
                            );
                        }
                        _ => {
                            eprint!(" {}", &content);
                        }
                    }
                    if is_thumb(row_idx.into(), scrollbar) {
                        execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                    }
                }
            }
        }
        self.draw_status(w, "hit *q* to quit, *s* to toggle summary mode")
    }
}
