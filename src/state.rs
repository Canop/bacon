use {
    crate::*,
    anyhow::*,
    crossterm::{
        cursor, execute,
        style::{Attribute, Color::*, Print},
    },
    std::io::Write,
    termimad::{
        minimad::{Alignment, Composite},
        Area, CompoundStyle, MadSkin,
    },
};

/// contains the currently rendered state of the application,
/// including scroll level and the current report (if any)
pub struct AppState {
    /// project name: usually the name of the package directory
    /// but might change
    pub project_name: String,
    /// "check", "clippy", "test", "check_windows", etc.
    pub job_name: String,
    /// the lines of a computation in progress
    lines: Option<Vec<CommandOutputLine>>,
    /// result of a command, hopefully a report
    cmd_result: CommandResult,
    /// a report wrapped for the size of the console
    wrapped_report: Option<WrappedReport>,
    /// screen width
    width: u16,
    /// screen height
    height: u16,
    /// whether a computation is in progress
    computing: bool,
    /// whether the user wants wrapped lines
    pub wrap: bool,
    /// whether the user wants backtraces
    pub backtrace: bool,
    /// whether we should display only titles and locations
    summary: bool,
    reverse: bool,
    /// colors and styles used for status bar
    status_skin: MadSkin,
    /// number of lines hidden on top due to scroll
    scroll: usize,
    /// item_idx of the item which was on top on last draw
    top_item_idx: usize,
}

impl AppState {
    pub fn new(mission: &Mission) -> Result<Self> {
        let mut status_skin = MadSkin::default();
        status_skin
            .paragraph
            .set_fgbg(AnsiValue(252), AnsiValue(239));
        status_skin.italic = CompoundStyle::new(Some(AnsiValue(204)), None, Attribute::Bold.into());
        let (width, height) = termimad::terminal_size();
        Ok(Self {
            project_name: mission.package_name.clone(),
            job_name: mission.job_name.clone(),
            lines: None,
            cmd_result: CommandResult::None,
            wrapped_report: None,
            width,
            height,
            computing: true,
            summary: mission.settings.summary,
            wrap: mission.settings.wrap,
            backtrace: false,
            reverse: mission.settings.reverse,
            status_skin,
            scroll: 0,
            top_item_idx: 0,
        })
    }
}

impl AppState {
    pub fn add_line(&mut self, line: CommandOutputLine) {
        let auto_scroll = self.is_scroll_at_bottom();
        self.lines.get_or_insert_with(Vec::new).push(line);
        if auto_scroll {
            // if the user never scrolled, we'll stick to the bottom
            self.scroll_to_bottom();
        }
    }
    pub fn new_task(&self) -> Task {
        Task {
            backtrace: self.backtrace,
        }
    }
    pub fn take_lines(&mut self) -> Option<Vec<CommandOutputLine>> {
        self.lines.take()
    }
    pub fn has_report(&self) -> bool {
        matches!(self.cmd_result, CommandResult::Report(_))
    }
    pub fn set_result(&mut self, mut cmd_result: CommandResult) {
        if self.reverse {
            cmd_result.reverse();
        }
        match &cmd_result {
            CommandResult::Report(_) => {
                debug!("GOT REPORT");
            }
            CommandResult::Failure(_) => {
                debug!("GOT FAILURE");
            }
            CommandResult::None => {
                debug!("GOT NONE ???");
            }
        }
        if let CommandResult::Report(ref mut report) = cmd_result {
            // if the last line is empty, we remove it, to
            // avoid a useless empty line at the end
            if report
                .lines
                .last()
                .map_or(false, |line| line.content.is_blank())
            {
                report.lines.pop();
            }
        }
        // we keep the scroll when the number of lines didn't change
        let reset_scroll = self.cmd_result.lines_len() != cmd_result.lines_len();
        self.wrapped_report = None;
        self.cmd_result = cmd_result;
        self.computing = false;
        if reset_scroll {
            self.reset_scroll();
        }
    }
    pub fn computation_starts(&mut self) {
        self.computing = true;
    }
    pub fn computation_stops(&mut self) {
        self.computing = false;
    }
    fn scroll_to_top(&mut self) {
        self.scroll = 0;
        self.top_item_idx = 0;
    }
    fn scroll_to_bottom(&mut self) {
        let ch = self.content_height();
        let ph = self.page_height();
        self.scroll = if ch > ph { ch - ph - 1 } else { 0 };
        // we don't set top_item_idx - does it matter?
    }
    fn is_scroll_at_bottom(&self) -> bool {
        self.scroll + self.page_height() + 1 >= self.content_height()
    }
    fn reset_scroll(&mut self) {
        if self.reverse {
            self.scroll_to_bottom();
        } else {
            self.scroll_to_top();
        }
    }
    fn fix_scroll(&mut self) {
        self.scroll = fix_scroll(self.scroll, self.content_height(), self.page_height());
    }
    /// get the scroll value needed to go to the last item (if any)
    fn get_last_item_scroll(&self) -> usize {
        if let CommandResult::Report(ref report) = self.cmd_result {
            if let Some(wrapped_report) = self.wrapped_report.as_ref().filter(|_| self.wrap) {
                let sub_lines = wrapped_report
                    .sub_lines
                    .iter()
                    .filter(|line| {
                        !(self.summary && line.src_line_type(report) == LineType::Normal)
                    })
                    .enumerate();
                for (row_idx, sub_line) in sub_lines {
                    if sub_line.src_line(report).item_idx == self.top_item_idx {
                        return row_idx;
                    }
                }
            } else {
                let lines = report
                    .lines
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
        0
    }
    fn try_scroll_to_last_top_item(&mut self) {
        self.scroll = self.get_last_item_scroll();
        self.fix_scroll();
    }
    pub fn toggle_summary_mode(&mut self) {
        self.summary ^= true;
        self.try_scroll_to_last_top_item();
    }
    pub fn toggle_backtrace(&mut self) {
        self.backtrace ^= true;
    }
    pub fn toggle_wrap_mode(&mut self) {
        self.wrap ^= true;
        if self.wrapped_report.is_some() {
            self.try_scroll_to_last_top_item();
        }
    }
    fn content_height(&self) -> usize {
        if let CommandResult::Report(report) = &self.cmd_result {
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
        self.scroll = cmd.apply(self.scroll, self.content_height(), self.page_height());
    }
    pub fn scroll(&mut self, w: &mut W, cmd: ScrollCommand) -> Result<()> {
        self.apply_scroll_command(cmd);
        self.draw(w)
    }
    fn draw_status(&self, w: &mut W, y: u16) -> Result<()> {
        let mut parts = vec!["hit *q* to quit"];
        if let CommandResult::Report(report) = &self.cmd_result {
            if report.suggest_backtrace {
                parts.push("*t* to toggle backtraces");
            } else {
                parts.push("*s* to toggle summary mode");
                if self.wrap {
                    parts.push("*w* to not wrap lines");
                } else {
                    parts.push("*w* to wrap lines");
                }
            }
        }
        let status = parts.join(", ");
        if self.height > 1 {
            goto(w, y)?;
            self.status_skin.write_composite_fill(
                w,
                Composite::from_inline(&status),
                self.width.into(),
                Alignment::Left,
            )?;
        }
        Ok(())
    }
    /// draw the line of colored badges, usually on top
    pub fn draw_badges(&mut self, w: &mut W, y: u16) -> Result<usize> {
        goto(w, y)?;
        let mut t_line = TLine::default();
        // white over grey
        t_line.add_badge(TString::badge(&self.project_name, 255, 240));
        // black over pink
        t_line.add_badge(TString::badge(&self.job_name, 235, 204));
        if let CommandResult::Report(report) = &self.cmd_result {
            let stats = &report.stats;
            if stats.errors > 0 {
                t_line.add_badge(TString::num_badge(stats.errors, "error", 235, 9));
            }
            if stats.test_fails > 0 {
                t_line.add_badge(TString::num_badge(stats.test_fails, "fail", 235, 208));
            }
            if stats.warnings > 0 {
                t_line.add_badge(TString::num_badge(stats.warnings, "warning", 235, 11));
            }
            if stats.items() == 0 {
                t_line.add_badge(TString::badge("pass!", 254, 2));
            }
        } else if let CommandResult::Failure(failure) = &self.cmd_result {
            t_line.add_badge(TString::badge(
                &format!("Command error code: {}", failure.error_code),
                235,
                9,
            ));
        }
        let width = self.width as usize;
        let cols = t_line.draw_in(w, width)?;
        clear_line(w)?;
        Ok(cols)
    }
    /// draw "computing...", the error code if any, or a blank line
    pub fn draw_computing(&mut self, w: &mut W, y: u16) -> Result<()> {
        goto(w, y)?;
        let width = self.width as usize;
        if self.computing {
            write!(
                w,
                "\u{1b}[38;5;235m\u{1b}[48;5;204m{:^w$}\u{1b}[0m",
                "computing...",
                w = width
            )?;
        } else {
            clear_line(w)?;
        }
        Ok(())
    }
    /// draw the report or the lines of the current computation, between
    /// y and self.page_height()
    pub fn draw_content(&mut self, w: &mut W, y: u16) -> Result<()> {
        if self.height < 4 {
            return Ok(());
        }
        let width = self.width as usize;
        let mut area = Area::new(0, y, self.width, self.page_height() as u16);
        let content_height = self.content_height();
        let scrollbar = area.scrollbar(self.scroll, content_height);
        if scrollbar.is_some() {
            area.width -= 1;
        }
        let mut top_item_idx = None;
        let top = if self.reverse && self.page_height() > content_height {
            self.page_height() - content_height + 1
        } else {
            0
        };
        let top = area.top + top as u16;
        for y in area.top..top {
            goto(w, y)?;
            clear_line(w)?;
        }
        if let CommandResult::Report(report) = &self.cmd_result {
            if self.wrap {
                if self.wrapped_report.is_none() {
                    self.wrapped_report = Some(WrappedReport::new(report, self.width));
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
                    let y = row_idx + top;
                    goto(w, y)?;
                    if let Some(sub_line) = sub_lines.next() {
                        top_item_idx.get_or_insert_with(|| sub_line.src_line(report).item_idx);
                        sub_line.draw_line_type(w, report)?;
                        write!(w, " ")?;
                        sub_line.draw(w, report)?;
                    }
                    clear_line(w)?;
                    if is_thumb(y.into(), scrollbar) {
                        execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                    }
                }
            } else {
                // unwrapped report
                let mut lines = report
                    .lines
                    .iter()
                    .filter(|line| !(self.summary && line.line_type == LineType::Normal))
                    .skip(self.scroll);
                for row_idx in 0..area.height {
                    let y = row_idx + top;
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
                        if width > line_type.cols() + 1 {
                            content.draw_in(w, width - 1 - line_type.cols())?;
                        }
                    }
                    clear_line(w)?;
                    if is_thumb(y.into(), scrollbar) {
                        execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                    }
                }
            }
            self.top_item_idx = top_item_idx.unwrap_or(0);
        } else {
            // report hasn't yet been computed or there was an error in command
            let lines = match &self.cmd_result {
                CommandResult::Failure(failure) => Some(&failure.lines),
                _ => self.lines.as_ref(),
            };
            if let Some(lines) = lines {
                for row_idx in 0..area.height {
                    let y = row_idx + top;
                    goto(w, y)?;
                    if let Some(line) = lines.get(row_idx as usize + self.scroll) {
                        line.content.draw_in(w, width)?;
                    }
                    clear_line(w)?;
                    if is_thumb(y.into(), scrollbar) {
                        execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                    }
                }
            }
        }
        Ok(())
    }
    /// draw the state on the whole terminal
    pub fn draw(&mut self, w: &mut W) -> Result<()> {
        if self.reverse {
            self.draw_status(w, 0)?;
            self.draw_content(w, 1)?;
            self.draw_computing(w, self.height - 2)?;
            self.draw_badges(w, self.height - 1)?;
        } else {
            self.draw_badges(w, 0)?;
            self.draw_computing(w, 1)?;
            self.draw_content(w, 2)?;
            self.draw_status(w, self.height - 1)?;
        }
        w.flush()?;
        Ok(())
    }
}
