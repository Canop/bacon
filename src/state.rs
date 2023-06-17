use {
    crate::*,
    anyhow::Result,
    std::io::Write,
    termimad::crossterm::{
        cursor,
        execute,
        style::{
            Attribute,
            Color::*,
            Print,
        },
    },
    termimad::{
        minimad::{
            Alignment,
            Composite,
        },
        Area,
        CompoundStyle,
        MadSkin,
    },
};

/// contains the currently rendered state of the application,
/// including scroll position and the current report (if any)
pub struct AppState<'s> {
    /// the mission to run, with settings
    pub mission: Mission<'s>,
    /// the lines of a computation in progress
    output: Option<CommandOutput>,
    /// wrapped output for the width of the console
    wrapped_output: Option<WrappedCommandOutput>,
    /// result of a command, hopefully a report
    pub cmd_result: CommandResult,
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
    /// whether we display the gui bottom-to-top
    reverse: bool,
    /// colors and styles used for status bar
    status_skin: MadSkin,
    /// number of lines hidden on top due to scroll
    scroll: usize,
    /// item_idx of the item which was on top on last draw
    top_item_idx: usize,
    /// the tool building the help line
    help_line: HelpLine,
    /// the help page displayed over the rest, if any
    help_page: Option<HelpPage>,
    /// display the raw output instead of the report
    raw_output: bool,
}

impl<'s> AppState<'s> {
    pub fn new(mission: Mission<'s>) -> Result<Self> {
        let mut status_skin = MadSkin::default();
        status_skin
            .paragraph
            .set_fgbg(AnsiValue(252), AnsiValue(239));
        status_skin.italic = CompoundStyle::new(Some(AnsiValue(204)), None, Attribute::Bold.into());
        let (width, height) = termimad::terminal_size();
        let help_line = HelpLine::new(mission.settings);
        Ok(Self {
            output: None,
            wrapped_output: None,
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
            help_line,
            help_page: None,
            mission,
            raw_output: false,
        })
    }

    pub fn add_line(
        &mut self,
        line: CommandOutputLine,
    ) {
        let auto_scroll = self.is_scroll_at_bottom();
        if let Some(output) = self.output.as_mut() {
            output.push(line);
            if auto_scroll {
                // if the user never scrolled, we'll stick to the bottom
                self.scroll_to_bottom();
            }
        } else {
            self.wrapped_output = None;
            self.output = {
                let mut output = CommandOutput::default();
                output.push(line);
                Some(output)
            };
            self.scroll = 0;
            self.fix_scroll();
        }
    }
    pub fn new_task(&self) -> Task {
        Task {
            backtrace: self.backtrace,
        }
    }
    pub fn take_output(&mut self) -> Option<CommandOutput> {
        self.wrapped_output = None;
        self.output.take()
    }
    pub fn has_report(&self) -> bool {
        matches!(self.cmd_result, CommandResult::Report(_))
    }
    pub fn toggle_raw_output(&mut self) {
        self.raw_output ^= true;
    }
    pub fn set_result(
        &mut self,
        mut cmd_result: CommandResult,
    ) {
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
        // we update the location file if required
        if self.mission.settings.export.enabled {
            if let Err(err) = cmd_result.update_location_file(&self.mission) {
                // FIXME user info
                error!("Error writing location file: {}", err);
            }
        }
        // we keep the scroll when the number of lines didn't change
        let reset_scroll = self.cmd_result.lines_len() != cmd_result.lines_len();
        self.wrapped_report = None;
        self.wrapped_output = None;
        self.cmd_result = cmd_result;
        self.computing = false;
        if reset_scroll {
            self.reset_scroll();
        }
        self.raw_output = false;
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
    pub fn keybindings(&self) -> &KeyBindings {
        &self.mission.settings.keybindings
    }
    fn try_scroll_to_last_top_item(&mut self) {
        self.scroll = self.get_last_item_scroll();
        self.fix_scroll();
    }
    /// close the help and return true if it was open,
    /// return false otherwise
    pub fn close_help(&mut self) -> bool {
        if self.help_page.is_some() {
            self.help_page = None;
            true
        } else {
            false
        }
    }
    pub fn is_help(&self) -> bool {
        self.help_page.is_some()
    }
    pub fn toggle_help(&mut self) {
        self.help_page = match self.help_page {
            Some(_) => None,
            None => Some(HelpPage::new(self.mission.settings)),
        };
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
            if self.mission.is_success(report) || self.raw_output {
                if let Some(wrapped_output) = self.wrapped_output.as_ref() {
                    wrapped_output.sub_lines.len()
                } else {
                    report.output.len()
                }
            } else {
                if let Some(wrapped_report) = self.wrapped_report.as_ref() {
                    wrapped_report.content_height(self.summary)
                } else {
                    report.stats.lines(self.summary)
                }
            }
        } else if let Some(output) = self.cmd_result.output().or(self.output.as_ref()) {
            match (self.wrap, self.wrapped_output.as_ref()) {
                (true, Some(wrapped_output)) => wrapped_output.sub_lines.len(),
                _ => output.len(),
            }
        } else {
            0
        }
    }
    fn page_height(&self) -> usize {
        self.height.max(3) as usize - 3
    }
    pub fn resize(
        &mut self,
        width: u16,
        height: u16,
    ) {
        if self.width != width {
            self.wrapped_report = None;
            self.wrapped_output = None;
        }
        self.width = width;
        self.height = height;
        self.try_scroll_to_last_top_item();
    }
    pub fn apply_scroll_command(
        &mut self,
        cmd: ScrollCommand,
    ) {
        if let Some(help_page) = self.help_page.as_mut() {
            help_page.apply_scroll_command(cmd);
        } else {
            debug!("content_height: {}", self.content_height());
            debug!("page_height: {}", self.page_height());
            self.scroll = cmd.apply(self.scroll, self.content_height(), self.page_height());
        }
    }
    /// draw the grey line containing the keybindings indications
    fn draw_help_line(
        &self,
        w: &mut W,
        y: u16,
    ) -> Result<()> {
        let markdown = self.help_line.markdown(self);
        if self.height > 1 {
            goto(w, y)?;
            self.status_skin.write_composite_fill(
                w,
                Composite::from_inline(&markdown),
                self.width.into(),
                Alignment::Left,
            )?;
        }
        Ok(())
    }
    /// draw the line of colored badges, usually on top
    pub fn draw_badges(
        &mut self,
        w: &mut W,
        y: u16,
    ) -> Result<usize> {
        goto(w, y)?;
        let mut t_line = TLine::default();
        // white over grey
        let project_name = &self.mission.location_name;
        t_line.add_badge(TString::badge(project_name, 255, 240));
        // black over pink
        t_line.add_badge(TString::badge(&self.mission.job_name, 235, 204));
        if let CommandResult::Report(report) = &self.cmd_result {
            let stats = &report.stats;
            if stats.errors > 0 {
                t_line.add_badge(TString::num_badge(stats.errors, "error", 235, 9));
            }
            if stats.test_fails > 0 {
                t_line.add_badge(TString::num_badge(stats.test_fails, "fail", 235, 208));
            } else if stats.passed_tests > 0 {
                t_line.add_badge(TString::badge("pass!", 254, 2));
            }
            if stats.warnings > 0 {
                t_line.add_badge(TString::num_badge(stats.warnings, "warning", 235, 11));
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
    pub fn draw_computing(
        &mut self,
        w: &mut W,
        y: u16,
    ) -> Result<()> {
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
    /// the action to execute now
    pub fn action(&self) -> Option<&Action> {
        self.mission
            .on_success()
            .as_ref()
            .filter(|_| self.cmd_result.is_success())
    }
    fn report_to_draw(&self) -> Option<&Report> {
        self.cmd_result
            .report()
            .filter(|_| !self.raw_output)
            .filter(|report| !self.mission.is_success(report))
    }
    fn update_wrap(
        &mut self,
        width: u16,
    ) {
        if let Some(report) = self.report_to_draw() {
            if self.wrapped_report.is_none() {
                self.wrapped_report = Some(WrappedReport::new(report, width));
                self.scroll = self.get_last_item_scroll();
            }
        } else if let Some(output) = self.cmd_result.output().or(self.output.as_ref()) {
            match self.wrapped_output.as_mut() {
                None => {
                    self.wrapped_output = Some(WrappedCommandOutput::new(output, width));
                    self.scroll = 0;
                }
                Some(wo) => {
                    wo.update(output, width);
                }
            }
        }
    }
    /// draw the report or the lines of the current computation, between
    /// y and self.page_height()
    pub fn draw_content(
        &mut self,
        w: &mut W,
        y: u16,
    ) -> Result<()> {
        if self.height < 4 {
            return Ok(());
        }
        let area = Area::new(0, y, self.width - 1, self.page_height() as u16);
        if self.wrap {
            self.update_wrap(area.width); // must be done before call to content_height
        }
        let content_height = self.content_height();
        let scrollbar = area.scrollbar(self.scroll, content_height);
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
        let width = self.width as usize;
        if let Some(report) = self.report_to_draw() {
            match (self.wrap, self.wrapped_report.as_ref()) {
                (true, Some(wrapped_report)) => {
                    // wrapped report
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
                            sub_line.draw(w, &report.lines)?;
                        }
                        clear_line(w)?;
                        if is_thumb(y.into(), scrollbar) {
                            execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                        }
                    }
                }
                _ => {
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
            }
            self.top_item_idx = top_item_idx.unwrap_or(0);
        } else if let Some(output) = self.cmd_result.output().or(self.output.as_ref()) {
            match (self.wrap, self.wrapped_output.as_ref()) {
                (true, Some(wrapped_output)) => {
                    let mut sub_lines = wrapped_output.sub_lines.iter().skip(self.scroll);
                    for row_idx in 0..area.height {
                        let y = row_idx + top;
                        goto(w, y)?;
                        if let Some(sub_line) = sub_lines.next() {
                            sub_line.draw(w, &output.lines)?;
                        }
                        clear_line(w)?;
                        if is_thumb(y.into(), scrollbar) {
                            execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
                        }
                    }
                }
                _ => {
                    let lines = &output.lines;
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
        }
        Ok(())
    }
    /// draw the state on the whole terminal
    pub fn draw(
        &mut self,
        w: &mut W,
    ) -> Result<()> {
        if self.reverse {
            self.draw_help_line(w, 0)?;
            if let Some(help_page) = self.help_page.as_mut() {
                help_page.draw(w, Area::new(0, 1, self.width, self.height - 1))?;
            } else {
                self.draw_content(w, 1)?;
                self.draw_computing(w, self.height - 2)?;
                self.draw_badges(w, self.height - 1)?;
            }
        } else {
            if let Some(help_page) = self.help_page.as_mut() {
                help_page.draw(w, Area::new(0, 0, self.width, self.height - 1))?;
            } else {
                self.draw_badges(w, 0)?;
                self.draw_computing(w, 1)?;
                self.draw_content(w, 2)?;
            }
            self.draw_help_line(w, self.height - 1)?;
        }
        w.flush()?;
        Ok(())
    }
}
