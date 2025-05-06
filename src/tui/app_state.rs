use {
    crate::*,
    anyhow::Result,
    crokey::KeyCombination,
    std::{
        io::Write,
        process::ExitStatus,
        time::Instant,
    },
    termimad::{
        Area,
        CompoundStyle,
        MadSkin,
        crossterm::{
            cursor,
            execute,
            style::{
                Attribute,
                Color::*,
                Print,
            },
        },
        minimad::{
            Alignment,
            Composite,
        },
    },
};

/// Currently rendered state of the TUI application
pub struct AppState<'s> {
    /// the mission to run, with settings
    pub mission: Mission<'s>,
    report_maker: ReportMaker,
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
    /// the optional RUST_BACKTRACE env var to set
    pub backtrace: Option<&'static str>,
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
    help_line: Option<HelpLine>,
    /// the help page displayed over the rest, if any
    help_page: Option<HelpPage>,
    /// display the raw output instead of the report
    raw_output: bool,
    /// whether auto-refresh is enabled
    pub auto_refresh: AutoRefresh,
    /// How many watch events were received since last job start
    pub changes_since_last_job_start: usize,
    /// whether to display the count of changes
    pub show_changes_count: bool,
    /// messages to display to the user for a short duration
    pub messages: Vec<Message>,
    /// the search state
    pub search: SearchState,
}

impl<'s> AppState<'s> {
    pub fn new(
        mission: Mission<'s>,
        headless: bool,
    ) -> Result<Self> {
        let report_maker = ReportMaker::new(&mission);
        let mut status_skin = MadSkin::default();
        status_skin
            .paragraph
            .set_fgbg(AnsiValue(252), AnsiValue(239));
        status_skin.italic = CompoundStyle::new(Some(AnsiValue(204)), None, Attribute::Bold.into());
        let (width, height) = if headless {
            (50, 50)
        } else {
            termimad::terminal_size()
        };
        let help_line = mission
            .settings
            .help_line
            .then(|| HelpLine::new(mission.settings));
        Ok(Self {
            report_maker,
            output: None,
            wrapped_output: None,
            cmd_result: CommandResult::None,
            wrapped_report: None,
            width,
            height,
            computing: true,
            summary: mission.settings.summary,
            wrap: mission.settings.wrap,
            backtrace: None,
            reverse: mission.settings.reverse,
            show_changes_count: mission.job.show_changes_count(),
            status_skin,
            scroll: 0,
            top_item_idx: 0,
            help_line,
            help_page: None,
            mission,
            raw_output: false,
            auto_refresh: AutoRefresh::Enabled,
            changes_since_last_job_start: 0,
            messages: Vec::new(),
            search: Default::default(),
        })
    }
    pub fn focus_file(
        &mut self,
        ffc: &FocusFileCommand,
    ) {
        if let CommandResult::Report(report) = &mut self.cmd_result {
            report.focus_file(ffc);
            if self.wrap {
                self.wrapped_report = None;
                self.update_wrap(self.width - 1);
            }
            self.reset_scroll();
        }
    }
    pub fn focus_search(&mut self) {
        self.search.focus_with_mode(SearchMode::Pattern);
        self.show_selected_found();
    }
    pub fn focus_goto(&mut self) {
        self.search.focus_with_mode(SearchMode::ItemIdx);
        self.show_selected_found();
    }
    // Handle the "back" operation, return true if it did (thus consuming the action)
    pub fn back(&mut self) -> bool {
        if self.search.focused() {
            self.search.unfocus_and_clear();
            true
        } else if self.help_page.is_some() {
            self.help_page = None;
            true
        } else if self.search.input_has_content() {
            self.search.clear();
            true
        } else {
            false
        }
    }
    pub fn copy_unstyled_output(&mut self) {
        let message = {
            #[cfg(feature = "clipboard")]
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => {
                    let mut content = String::new();
                    for line in self.lines_to_draw() {
                        content.push_str(&line.content.to_raw());
                        content.push('\n');
                    }
                    let _ = clipboard.set_text(content);
                    "Output copied to clipboard"
                }
                Err(e) => {
                    error!("Failed to copy output to clipboard: {}", e);
                    "Clipboard error - nothing copied"
                }
            }
            #[cfg(not(feature = "clipboard"))]
            "clipboard feature not enabled : nothing copied"
        };
        self.messages.push(Message::short(message));
    }
    pub fn next_match(&mut self) {
        self.search.next_match();
        self.show_selected_found();
    }
    pub fn previous_match(&mut self) {
        self.search.previous_match();
        self.show_selected_found();
    }
    // Handle the "validate" operation, return true if it did (thus consuming the action)
    pub fn validate(&mut self) -> bool {
        if self.search.focused() {
            self.search.unfocus();
            true
        } else {
            false
        }
    }
    pub fn has_search(&self) -> bool {
        self.search.focused() || self.search.input_has_content()
    }
    /// handle a raw, uninterpreted key combination (in an input if there's one
    /// focused), return true if the key was consumed (if not, keybindings will
    /// be computed)
    pub fn apply_key_combination(
        &mut self,
        key: KeyCombination,
    ) -> bool {
        if self.search.apply_key_combination(key) {
            self.update_search();
            self.show_selected_found();
            return true;
        }
        false
    }
    pub fn update_search(&mut self) {
        if self.search.is_up_to_date() {
            return;
        }
        let founds = if self.search.input_has_content() {
            let search = self.search.search();
            let lines = self.lines_to_draw();
            search.search_lines(lines)
        } else {
            Vec::new()
        };
        self.search.set_founds(founds);
    }
    /// Do a partial update for some potential added lines
    pub fn update_search_from_line(
        &mut self,
        line_count_before: usize,
    ) {
        if !self.search.input_has_content() {
            return;
        }
        // it's probably fine now to search without taking filtering into
        // account as we're only adding lines in the raw output where there's
        // no filtering
        let lines = self.lines_to_draw_unfiltered();
        let search = self.search.search();
        let new_founds = search.search_lines(&lines[line_count_before..]);
        self.search.extend_founds(new_founds);
    }
    pub fn add_line(
        &mut self,
        line: CommandOutputLine,
    ) {
        let auto_scroll = self.is_scroll_at_bottom();
        let line_count_before = self.lines_to_draw_unfiltered().len();
        if let Some(output) = self.output.as_mut() {
            self.report_maker.receive_line(line, output);
            if self.wrap {
                self.update_wrap(self.width - 1);
            }
            if auto_scroll {
                // if the user never scrolled, we'll stick to the bottom
                self.scroll_to_bottom();
            }
        } else {
            self.wrapped_output = None;
            self.output = {
                let mut output = CommandOutput::default();
                self.report_maker.receive_line(line, &mut output);
                Some(output)
            };
            self.scroll = 0;
            self.fix_scroll();
        }
        self.update_search_from_line(line_count_before);
    }
    pub fn new_task(&self) -> Task {
        Task {
            backtrace: self.backtrace,
            grace_period: self.mission.job.grace_period(),
        }
    }
    pub fn take_output(&mut self) -> Option<CommandOutput> {
        self.search.touch();
        self.wrapped_output = None;
        self.output.take()
    }
    pub fn has_report(&self) -> bool {
        matches!(self.cmd_result, CommandResult::Report(_))
    }
    pub fn can_be_scoped(&self) -> bool {
        self.cmd_result
            .report()
            .is_some_and(|report| report.stats.can_scope_tests())
    }
    pub fn failures_scope(&self) -> Option<Scope> {
        if !self.can_be_scoped() {
            return None;
        }
        self.cmd_result.report().map(|report| Scope {
            tests: report.failure_keys.clone(),
        })
    }
    pub fn toggle_raw_output(&mut self) {
        self.raw_output ^= true;
        if self.wrapped_output.is_some() {
            self.wrapped_output = None;
        }
        if self.wrap {
            self.update_wrap(self.width - 1);
        }
        self.search.touch();
    }
    pub fn finish_task(
        &mut self,
        exit_status: Option<ExitStatus>,
    ) -> Result<()> {
        let output = self.take_output().unwrap_or_default();
        let result = self.report_maker.build_result(output, exit_status)?;
        self.set_result(result);
        Ok(())
    }
    fn set_result(
        &mut self,
        mut cmd_result: CommandResult,
    ) {
        self.search.touch();
        if self.reverse {
            cmd_result.reverse();
        }
        match &cmd_result {
            CommandResult::Report(report) => {
                debug!("Got report - Stats: {:#?}", report.stats);
            }
            CommandResult::Failure(_) => {
                debug!("Got failure");
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
                .is_some_and(|line| line.content.is_blank())
            {
                report.lines.pop();
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
        if self.wrap {
            self.update_wrap(self.width - 1);
        }

        // we do all exports which are set to auto
        self.mission.settings.exports.do_auto_exports(self);
    }
    pub fn is_computing(&self) -> bool {
        self.computing
    }
    pub fn clear(&mut self) {
        debug!("state.clear");
        self.take_output();
        self.cmd_result = CommandResult::None;
        self.search.touch();
    }
    /// Start a new task on the current mission
    pub fn start_computation(
        &mut self,
        executor: &mut MissionExecutor,
    ) -> Result<TaskExecutor> {
        debug!("state.start_computation");
        self.computation_starts();
        executor.start(self.new_task())
    }
    /// Called when a task has started
    pub fn computation_starts(&mut self) {
        if !self.mission.job.background() {
            self.clear();
        }
        self.report_maker.start(&self.mission);
        self.computing = true;
        self.changes_since_last_job_start = 0;
        self.search.touch();
    }
    pub fn computation_stops(&mut self) {
        self.computing = false;
    }
    pub fn receive_watch_event(&mut self) {
        self.changes_since_last_job_start += 1;
    }
    fn scroll_to_top(&mut self) {
        self.scroll = 0;
        self.top_item_idx = 0;
    }
    fn scroll_to_bottom(&mut self) {
        let ch = self.content_height();
        let ph = self.page_height();
        self.scroll = ch.saturating_sub(ph);
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
        let lines = self.lines_to_draw();
        for (row_idx, line) in lines.enumerate() {
            if line.item_idx == self.top_item_idx {
                return row_idx;
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
    fn show_line(
        &mut self,
        line_idx: usize,
    ) {
        let page_height = self.page_height();
        if line_idx < self.scroll || line_idx >= self.scroll + page_height {
            self.scroll = (line_idx - (page_height / 2).min(line_idx))
                .min(self.content_height().max(page_height) - page_height);
        }
    }
    fn show_selected_found(&mut self) {
        if let Some(selected_line) = self.search.selected_found_line() {
            self.show_line(selected_line);
        }
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
        self.search.touch();
        self.update_search();
        self.show_selected_found();
    }
    pub fn toggle_backtrace(
        &mut self,
        level: &'static str,
    ) {
        self.backtrace = if self.backtrace == Some(level) {
            None
        } else {
            Some(level)
        };
    }
    pub fn toggle_wrap_mode(&mut self) {
        self.wrap ^= true;
        if self.wrapped_output.is_some() {
            self.wrapped_output = None;
        }
        if self.wrap {
            self.update_wrap(self.width - 1);
        }
        if self.wrapped_report.is_some() {
            self.try_scroll_to_last_top_item();
        }
        self.search.touch();
    }
    fn content_height(&self) -> usize {
        let lines = self.lines_to_draw();
        lines.count()
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
        if self.wrap {
            self.update_wrap(self.width - 1);
        }
        self.try_scroll_to_last_top_item();
        self.search.touch();
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
    fn draw_status_line(
        &mut self,
        w: &mut W,
        y: u16,
    ) -> Result<()> {
        let mut help_start = 0;
        // Search input
        if self.search.must_be_drawn() {
            let search_width = (self.width / 4).clamp(9, 27);
            self.search.draw_prefixed_input(w, 0, y, search_width)?;
            help_start += search_width;
        }
        goto(w, help_start, y)?;
        // Help line
        if let Some(help_line) = &self.help_line {
            let markdown = help_line.markdown(self);
            if self.height > 1 {
                let help_width = self.width - help_start;
                self.status_skin.write_composite_fill(
                    w,
                    Composite::from_inline(&markdown),
                    help_width.into(),
                    Alignment::Left,
                )?;
            }
        } else {
            clear_line(w)?;
        }
        Ok(())
    }

    pub fn job_badges(&self) -> Vec<TString> {
        let mut badges = Vec::new();
        let project_name = &self.mission.location_name;
        badges.push(TString::badge(project_name, 255, 240));
        let job_label = self.mission.concrete_job_ref.badge_label();
        badges.push(TString::badge(&job_label, 235, 204));
        if let CommandResult::Report(report) = &self.cmd_result {
            let stats = &report.stats;
            if stats.errors > 0 {
                badges.push(TString::num_badge(stats.errors, "error", 235, 9));
            }
            if stats.test_fails > 0 {
                badges.push(TString::num_badge(stats.test_fails, "fail", 235, 208));
            } else if stats.passed_tests > 0 {
                badges.push(TString::badge("pass!", 254, 2));
            }
            if stats.warnings > 0 {
                badges.push(TString::num_badge(stats.warnings, "warning", 235, 11));
            }
        } else if let CommandResult::Failure(failure) = &self.cmd_result {
            badges.push(TString::badge(
                &format!("Command error code: {}", failure.error_code),
                235,
                9,
            ));
        }
        badges
    }

    /// draw the line of colored badges, usually on top
    pub fn draw_badges(
        &mut self,
        w: &mut W,
        y: u16,
    ) -> Result<usize> {
        goto_line(w, y)?;
        let mut t_line = TLine::default();
        for badge in self.job_badges() {
            t_line.add_badge(badge);
        }
        if self.show_changes_count {
            t_line.add_badge(TString::num_badge(
                self.changes_since_last_job_start,
                "change",
                235,
                6,
            ));
        }
        self.search.add_summary_tstring(&mut t_line);
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
        goto_line(w, y)?;
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
    /// draw message
    pub fn draw_message(
        &mut self,
        w: &mut W,
        y: u16,
    ) -> Result<()> {
        let Some(message) = self.messages.first_mut() else {
            return Ok(());
        };
        if let Some(start) = message.display_start {
            if start.elapsed() > message.display_duration {
                self.messages.remove(0);
                return Ok(());
            }
        } else {
            message.display_start = Some(Instant::now());
        }
        goto_line(w, y)?;
        let markdown = format!(" {}", message.markdown);
        self.status_skin.write_composite_fill(
            w,
            Composite::from_inline(&markdown),
            self.width.into(),
            Alignment::Left,
        )?;
        Ok(())
    }
    pub fn is_success(&self) -> bool {
        match &self.cmd_result {
            CommandResult::Report(report) => self.mission.is_success(report),
            _ => false,
        }
    }
    pub fn is_failure(&self) -> bool {
        match &self.cmd_result {
            CommandResult::Report(report) => !self.mission.is_success(report),
            CommandResult::Failure(_) => true,
            CommandResult::None => false,
        }
    }
    /// Return the (unfiltered) set of lines to draw, depending on whether we wrap or not
    /// and whether we have a report or not.
    fn lines_to_draw_unfiltered(&self) -> &[Line] {
        if let Some(report) = self.report_to_draw() {
            match (self.wrap, self.wrapped_report.as_ref()) {
                (true, Some(wrapped_report)) => {
                    // wrapped report
                    &wrapped_report.sub_lines
                }
                _ => {
                    // unwrapped report
                    &report.lines
                }
            }
        } else if let Some(output) = self.cmd_result.output().or(self.output.as_ref()) {
            match (self.wrap, self.wrapped_output.as_ref()) {
                (true, Some(wrapped_output)) => {
                    // wrapped raw command output
                    &wrapped_output.sub_lines
                }
                _ => {
                    // unwrapped raw command output
                    &output.lines
                }
            }
        } else {
            // nothing yet
            &[]
        }
    }
    fn lines_to_draw(&self) -> impl Iterator<Item = &Line> {
        self.lines_to_draw_unfiltered().iter().filter(|line| {
            // if this command failed, always show the output
            matches!(self.cmd_result, CommandResult::Failure(..)) || line.matches(self.summary)
        })
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
                    self.reset_scroll();
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
        let content_height = self.content_height();
        let scrollbar = area.scrollbar(self.scroll, content_height);
        let mut top_item_idx = None;
        let top = if self.reverse && self.page_height() > content_height {
            self.page_height() - content_height
        } else {
            0
        };
        let top = area.top + top as u16;
        for y in area.top..top {
            goto_line(w, y)?;
            clear_line(w)?;
        }
        let width = self.width as usize;
        let lines = self.lines_to_draw();
        let mut lines = lines.enumerate().skip(self.scroll);
        let mut found_idx = 0;
        #[derive(Debug)]
        struct PendingContinuation {
            trange: TRange,
            style: &'static str,
        }
        let mut pending_continuation = None;
        for row_idx in 0..area.height {
            let y = row_idx + top;
            goto_line(w, y)?;
            if let Some((line_idx, line)) = lines.next() {
                top_item_idx.get_or_insert(line.item_idx);
                line.line_type.draw(w, line.item_idx)?;
                write!(w, " ")?;
                if width > line.line_type.cols() + 1 {
                    let mut tline = &line.content;

                    // search for the optional founds related to that line
                    let mut line_founds = Vec::new();
                    let found_idx_before_line = found_idx;
                    let founds = self.search.founds();
                    while found_idx < founds.len() {
                        let found = &founds[found_idx];
                        if found.line_idx > line_idx {
                            break;
                        }
                        if found.line_idx == line_idx {
                            line_founds.push(found);
                        }
                        found_idx += 1;
                    }

                    // apply the modification on the tline
                    let mut modified;
                    let previous_continuation = pending_continuation.take();
                    if previous_continuation.is_some() || !line_founds.is_empty() {
                        modified = tline.clone();
                        // We iterate on founds in reverse, so that we change the tline from
                        // the end, so that the tstring index in the founds stay valid when
                        // tstrings are added by the change_range_style method.
                        for (in_line_idx, found) in line_founds.iter().enumerate().rev() {
                            let cur_idx = found_idx_before_line + in_line_idx;
                            let style = if self.search.selected_found() == cur_idx {
                                CSI_FOUND_SELECTED
                            } else {
                                CSI_FOUND
                            };
                            modified.change_range_style(found.trange, style.to_string());
                            if let Some(continued) = &found.continued {
                                pending_continuation = Some(PendingContinuation {
                                    trange: *continued,
                                    style,
                                });
                            }
                        }
                        if let Some(continuation) = previous_continuation {
                            modified.change_range_style(
                                continuation.trange,
                                continuation.style.to_string(),
                            );
                        }
                        tline = &modified;
                    }
                    tline.draw_in(w, width - 1 - line.line_type.cols())?;
                }
            }
            clear_line(w)?;
            if is_thumb(y.into(), scrollbar) {
                execute!(w, cursor::MoveTo(area.width, y), Print('▐'.to_string()))?;
            }
        }
        Ok(())
    }
    /// draw the state on the whole terminal
    pub fn draw(
        &mut self,
        w: &mut W,
    ) -> Result<()> {
        self.update_search();
        if self.reverse {
            self.draw_status_line(w, 0)?;
            if let Some(help_page) = self.help_page.as_mut() {
                help_page.draw(w, Area::new(0, 1, self.width, self.height - 1))?;
            } else {
                self.draw_content(w, 1)?;
                self.draw_computing(w, self.height - 2)?;
                self.draw_message(w, self.height - 2)?;
                self.draw_badges(w, self.height - 1)?;
            }
        } else {
            if let Some(help_page) = self.help_page.as_mut() {
                help_page.draw(w, Area::new(0, 0, self.width, self.height - 1))?;
            } else {
                self.draw_badges(w, 0)?;
                self.draw_computing(w, 1)?;
                self.draw_message(w, 1)?; // drawn over the "computing..." line
                self.draw_content(w, 2)?;
            }
            self.draw_status_line(w, self.height - 1)?;
        }
        w.flush()?;
        Ok(())
    }
}
