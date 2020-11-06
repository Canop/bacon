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
    /// project name
    pub name: String,
    /// the raw lines of a computation in progress
    lines: Option<Vec<String>>,
    /// a totally computed report
    report: Option<Report>,
    /// screen width
    width: u16,
    /// screen height
    height: u16,
    /// whether a computation is in progress
    pub computing: bool,
    /// whether we should display only titles and locations
    summary: bool,
    /// colors and styles used for status bar
    status_skin: MadSkin,
    /// number of lines hidden on top due to scroll
    scroll: usize,
    /// there was at least a scroll operation
    scrolled: bool,
}
impl AppState {
    pub fn new(root_dir: &Path) -> Result<Self> {
        let mut status_skin = MadSkin::default();
        status_skin.paragraph.set_bg(DarkGrey);
        status_skin.italic = CompoundStyle::with_fg(Yellow);
        let (width, height) = termimad::terminal_size();
        Ok(Self {
            name: root_dir.file_name().unwrap().to_string_lossy().to_string(),
            lines: None,
            report: None,
            width,
            height,
            computing: true,
            summary: false,
            status_skin,
            scroll: 0,
            scrolled: false,
        })
    }
}

impl AppState {
    pub fn add_line(&mut self, line: String) {
        self.lines.get_or_insert_with(Vec::new).push(line);
        if !self.scrolled {
            // if the user never scrolled, we'll stick to the bottom
            self.apply_scroll_command(ScrollCommand::Bottom);
        }
    }
    pub fn take_lines(&mut self) -> Option<Vec<String>> {
        self.lines.take()
    }
    pub fn set_report(&mut self, report: Report) {
        if self.report.as_ref()
            .map_or(true, |old_report| old_report.lines.len() != report.lines.len())
        {
            // we keep the scroll when the number of lines didn't change
            self.scroll = 0;
        }
        self.report = Some(report);
    }
    fn fix_scroll(&mut self) {
        self.scroll = fix_scroll(self.scroll, self.content_height(), self.page_height());
    }
    pub fn get_current_top_item_idx(&self) -> Option<usize> {
        self.report.as_ref()
            .and_then(|report| {
                report.lines
                    .iter()
                    .filter(|line| !(self.summary && line.line_type == LineType::Normal))
                    .skip(self.scroll)
                    .next()
            })
            .map(|line| line.item_idx)
    }
    fn try_set_top_item(&mut self, item_idx: usize) {
        if self.get_current_top_item_idx() != Some(item_idx) {
            if let Some(report) = self.report.as_ref() {
                let lines = report.lines
                    .iter()
                    .filter(|line| !(self.summary && line.line_type == LineType::Normal))
                    .enumerate();
                for (row_idx, line) in lines {
                    if line.item_idx == item_idx {
                        self.scroll = row_idx;
                        break;
                    }
                }
            }
        }
        self.fix_scroll();
    }
    pub fn toggle_summary_mode(&mut self) {
        let item_idx = self.get_current_top_item_idx();
        self.summary ^= true;
        if let Some(item_idx) = item_idx {
            self.try_set_top_item(item_idx);
        }
    }
    fn content_height(&self) -> usize {
        if let Some(report) = &self.report {
            report.stats.lines(self.summary)
        } else if let Some(lines) = &self.lines {
            lines.len()
        } else {
            0
        }
    }
    fn page_height(&self) -> usize {
        self.height.max(3) as usize - 3
    }
    pub fn resize(&mut self, width: u16, height: u16) {
        let item_idx = self.get_current_top_item_idx();
        self.width = width;
        self.height = height;
        if let Some(item_idx) = item_idx {
            self.try_set_top_item(item_idx);
        }
    }
    fn apply_scroll_command(&mut self, cmd: ScrollCommand) {
        self.scroll = cmd.apply(
            self.scroll,
            self.content_height(),
            self.page_height(),
        );
    }
    pub fn scroll(&mut self, w: &mut W, cmd: ScrollCommand) -> Result<()> {
        debug!("user scroll command: {:?}", cmd);
        self.scrolled = true;
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
    /// draw the state on the whole terminal
    pub fn draw(&mut self, w: &mut W) -> Result<()> {
        let width = self.width as usize;
        goto(w, 0)?;
        //// colored badges on top
        write!(w, "{} ", format!(" {} ", &self.name).white().bold().on_dark_grey())?;
        if let Some(report) = &self.report {
            let stats = &report.stats;
            if stats.errors > 0 {
                let s = if stats.errors == 1 {
                    " 1 error ".to_string()
                } else {
                    format!(" {} errors ", stats.errors)
                };
                write!(w, "{} ", s.black().bold().on_red())?;
            }
            if stats.warnings > 0 {
                let s = if stats.warnings == 1 {
                    " 1 warning ".to_string()
                } else {
                    format!(" {} warnings ", stats.warnings)
                };
                write!(w, "{} ", s.black().bold().on_yellow())?;
            }
            if stats.items() == 0 {
                write!(w, "{} ", " pass! ".white().bold().on_dark_green())?;
            }
        }
        //// computing...
        goto(w, 1)?;
        if self.computing {
            write!(w, "{}", format!("{:^w$}", "computing...", w = width).white().on_dark_grey())?;
        }
        //// content
        if self.height < 4 {
            return self.draw_status(w, "terminal too small");
        }
        let mut area = Area::new(0, 2, self.width, self.page_height() as u16);
        let content_height = self.content_height();
        let scrollbar = area.scrollbar(self.scroll as i32, content_height as i32);
        if scrollbar.is_some() {
            area.width -= 1;
        }
        if let Some(report) = &self.report {
            // a totally computed report
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
                            write!(
                                w,
                                "{} {}",
                                format!("{:>2} ", item_idx).black().bold().on_red(),
                                &content,
                            )?;
                        }
                        LineType::Title(Kind::Warning) => {
                            write!(
                                w,
                                "{} {}",
                                format!("{:>2} ", item_idx).black().bold().on_yellow(),
                                &content,
                            )?;
                        }
                        _ => {
                            write!(w, " {}", &content)?;
                        }
                    }
                    if is_thumb(row_idx.into(), scrollbar) {
                        execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                    }
                }
            }
        } else if let Some(lines) = &self.lines {
            // initial computation
            for row_idx in 0..area.height {
                let y = row_idx + area.top;
                goto(w, y)?;
                if let Some(line) = lines.get(row_idx as usize + self.scroll) {
                    write!(w, "{}", line)?;
                }
                if is_thumb(row_idx.into(), scrollbar) {
                    execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                }
            }
        }
        //// Status bar
        let status = if self.report.is_some() {
            "hit *q* to quit, *s* to toggle summary mode"
        } else {
            "hit *q* to quit"
        };
        self.draw_status(w, status)
    }
}
