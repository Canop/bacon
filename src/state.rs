use {
    crate::*,
    anyhow::*,
    crossterm::{
        cursor, execute,
        style::{Attribute, Color::*, Colorize, Print, Styler},
    },
    minimad::{Alignment, Composite},
    std::io::Write,
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
    wrapped_report: Option<WrappedReport>,
    /// screen width
    width: u16,
    /// screen height
    height: u16,
    /// whether a computation is in progress
    pub computing: bool,
    /// whether the user wants wrapped lines
    pub wrap: bool,
    /// whether we should display only titles and locations
    summary: bool,
    /// colors and styles used for status bar
    status_skin: MadSkin,
    /// number of lines hidden on top due to scroll
    scroll: usize,
    /// auto-scroll to bottom on line addition
    auto_scroll: bool,
    /// item_idx of the item which was on top on last draw
    top_item_idx: usize,
}
impl AppState {
    pub fn new(mission: &Mission) -> Result<Self> {
        let mut status_skin = MadSkin::default();
        status_skin.paragraph.set_fgbg(AnsiValue(252), AnsiValue(239));
        //status_skin.italic = CompoundStyle::with_fg(AnsiValue(204));
        status_skin.italic = CompoundStyle::new(Some(AnsiValue(204)), None, Attribute::Bold.into());
        let (width, height) = termimad::terminal_size();
        Ok(Self {
            name: mission.name.clone(),
            lines: None,
            report: None,
            wrapped_report: None,
            width,
            height,
            computing: true,
            summary: mission.display_settings.summary,
            wrap: mission.display_settings.wrap,
            status_skin,
            scroll: 0,
            auto_scroll: true,
            top_item_idx: 0,
        })
    }
}

impl AppState {
    pub fn add_line(&mut self, line: String) {
        self.lines.get_or_insert_with(Vec::new).push(line);
        if self.auto_scroll {
            // if the user never scrolled, we'll stick to the bottom
            self.apply_scroll_command(ScrollCommand::Bottom);
        }
    }
    pub fn take_lines(&mut self) -> Option<Vec<String>> {
        self.lines.take()
    }
    pub fn has_report(&self) -> bool {
        self.report.is_some()
    }
    pub fn set_report(&mut self, report: Report) {
        if self.report.as_ref()
            .map_or(true, |old_report| old_report.lines.len() != report.lines.len())
        {
            // we keep the scroll when the number of lines didn't change
            self.scroll = 0;
            self.top_item_idx = 0;
        }
        self.report = Some(report);
        self.wrapped_report = None;
        self.auto_scroll = false;
    }
    fn fix_scroll(&mut self) {
        self.scroll = fix_scroll(self.scroll, self.content_height(), self.page_height());
    }
    fn get_last_item_scroll(&self) -> usize {
        if let Some(report) = self.report.as_ref() {
            if let Some(wrapped_report) = self.wrapped_report.as_ref() {
                let sub_lines = wrapped_report.sub_lines
                    .iter()
                    .filter(|line| {
                        !(self.summary && line.src_line_type(report) == LineType::Normal)
                    })
                    .enumerate();
                for (row_idx, sub_line) in sub_lines {
                    if sub_line.src_line(&report).item_idx == self.top_item_idx {
                        return row_idx;
                    }
                }
            } else {
                let lines = report.lines
                    .iter()
                    .filter(|line| !(self.summary && line.line_type == LineType::Normal))
                    .enumerate();
                for (row_idx, line) in lines {
                    if line.item_idx == self.top_item_idx {
                        return row_idx;
                    }
                }
            }
        }
        return 0;
    }
    fn try_scroll_to_last_top_item(&mut self) {
        self.scroll = self.get_last_item_scroll();
        self.fix_scroll();
    }
    pub fn toggle_summary_mode(&mut self) {
        self.summary ^= true;
        self.try_scroll_to_last_top_item();
    }
    pub fn toggle_wrap_mode(&mut self) {
        self.wrap ^= true;
        if self.wrapped_report.is_some() {
            self.try_scroll_to_last_top_item();
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
        if self.width != width {
            self.wrapped_report = None;
        }
        self.width = width;
        self.height = height;
        self.try_scroll_to_last_top_item();
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
        self.auto_scroll = false;
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
    fn draw_name(&self, w: &mut W) -> Result<()> {
        // white on grey
        //write!(w, "{} ", format!(" {} ", &self.name).white().bold().on_dark_grey())?;
        // black over pink
        write!(w, "\u{1b}[1m\u{1b}[38;5;235m\u{1b}[48;5;204m {} \u{1b}[0m ", &self.name)?;
        // pink over grey
        //write!(w, "\u{1b}[48;5;239m\u{1b}[1m\u{1b}[38;5;204m {} \u{1b}[0m ", &self.name)?;
        Ok(())
    }
    /// draw the state on the whole terminal
    pub fn draw(&mut self, w: &mut W) -> Result<()> {
        let width = self.width as usize;
        goto(w, 0)?;
        //// colored badges on top
        self.draw_name(w)?;
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
        let mut top_item_idx = None;
        if let Some(report) = &self.report {
            if self.wrap {
                if self.wrapped_report.is_none() {
                    self.wrapped_report = Some(WrappedReport::new(&report, self.width));
                    self.scroll = self.get_last_item_scroll();
                }
                // SAFETY: we just ensured it's here
                // (will be cleaner when Option::insert is available in stable)
                let wrapped_report = self.wrapped_report.as_ref().unwrap();
                let mut sub_lines = wrapped_report
                    .sub_lines
                    .iter()
                    .filter(|sub_line| {
                        !(self.summary && sub_line.src_line_type(report) == LineType::Normal)
                    })
                    .skip(self.scroll);
                for row_idx in 0..area.height {
                    let y = row_idx + area.top;
                    goto(w, y)?;
                    if let Some(sub_line) = sub_lines.next() {
                        top_item_idx.get_or_insert_with(|| {
                            sub_line.src_line(&report).item_idx
                        });
                        sub_line.draw_line_type(w, &report)?;
                        write!(w, " ")?;
                        sub_line.draw(w, &report)?;
                        if is_thumb(row_idx.into(), scrollbar) {
                            execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                        }
                    }
                }
            } else {
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
                        top_item_idx.get_or_insert(*item_idx);
                        line_type.draw(w, *item_idx)?;
                        write!(w, " ")?;
                        content.draw(w)?;
                        if is_thumb(row_idx.into(), scrollbar) {
                            execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                        }
                    }
                }
            }
            self.top_item_idx = top_item_idx.unwrap_or(0);
        } else if let Some(lines) = &self.lines {
            // initial computation - report hasn't yet been computed
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
            "hit *q* to quit, *s* to toggle summary mode, *w* to toggle wrapping"
        } else {
            "hit *q* to quit"
        };
        self.draw_status(w, status)
    }
}
