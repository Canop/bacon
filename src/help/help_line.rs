use crate::*;

pub struct HelpLine {
    quit: String,
    toggle_summary: Option<String>,
    wrap: Option<String>,
    not_wrap: Option<String>,
    toggle_backtrace: Option<String>,
    help: Option<String>,
    close_help: Option<String>,
    pause: Option<String>,
    unpause: Option<String>,
    scope: Option<String>,
    search: Option<String>,
    next_match: Option<String>,
    previous_match: Option<String>,
    clear_search: Option<String>,
    validate_search: Option<String>,
}

impl HelpLine {
    pub fn new(settings: &Settings) -> Self {
        let kb = &settings.keybindings;
        let quit = kb
            .shortest_internal_key(Action::Quit)
            .map(|k| format!("*{k}* to quit"))
            .expect("the app to be quittable");
        let toggle_summary = kb
            .shortest_internal_key(Action::ToggleSummary)
            .map(|k| format!("*{k}* to toggle summary mode"));
        let wrap = kb
            .shortest_internal_key(Action::ToggleWrap)
            .map(|k| format!("*{k}* to wrap lines"));
        let not_wrap = kb
            .shortest_internal_key(Action::ToggleWrap)
            .map(|k| format!("*{k}* to not wrap lines"));
        let toggle_backtrace = kb
            .shortest_action_key(|action| {
                matches!(action, Action::ToggleBacktrace(_))
            })
            .map(|k| format!("*{k}* to toggle backtraces"));
        let help = kb
            .shortest_internal_key(Action::Help)
            .map(|k| format!("*{k}* for help"));
        let close_help = kb
            .shortest_internal_key(Action::Back)
            .or_else(|| kb.shortest_internal_key(Action::Help))
            .map(|k| format!("*{k}* to close this help"));
        let pause = kb
            .shortest_internal_key(Action::Pause)
            .or(kb.shortest_internal_key(Action::TogglePause))
            .map(|k| format!("*{k}* to pause"));
        let unpause = kb
            .shortest_internal_key(Action::Unpause)
            .or(kb.shortest_internal_key(Action::TogglePause))
            .map(|k| format!("*{k}* to unpause"));
        let scope = kb
            .shortest_internal_key(Action::ScopeToFailures)
            .map(|k| format!("*{k}* to scope to failures"));
        let search = kb
            .shortest_internal_key(Action::FocusSearch)
            .map(|k| format!("*{k}* to search"));
        let next_match = kb
            .shortest_internal_key(Action::NextMatch)
            .map(|k| format!("*{k}* for next match"));
        let previous_match = kb
            .shortest_internal_key(Action::PreviousMatch)
            .map(|k| format!("*{k}* for previous match"));
        let clear_search = kb
            .shortest_internal_key(Action::Back)
            .map(|k| format!("*{k}* to clear"));
        let validate_search = kb
            .shortest_internal_key(Action::Validate)
            .map(|k| format!("*{k}* to validate"));
        Self {
            quit,
            toggle_summary,
            wrap,
            not_wrap,
            toggle_backtrace,
            help,
            close_help,
            pause,
            unpause,
            scope,
            search,
            next_match,
            previous_match,
            clear_search,
            validate_search,
        }
    }
    fn applicable_parts(
        &self,
        state: &AppState,
    ) -> Vec<&str> {
        let mut parts: Vec<&str> = Vec::new();
        if state.is_help() {
            parts.push(&self.quit);
            if let Some(s) = &self.close_help {
                parts.push(s);
            }
            return parts;
        }
        if state.has_search() {
            if let Some(s) = &self.next_match {
                parts.push(s);
            }
            if let Some(s) = &self.previous_match {
                parts.push(s);
            }
            if let Some(s) = &self.clear_search {
                parts.push(s);
            }
            if state.search.focused() {
                if let Some(s) = &self.validate_search {
                    parts.push(s);
                }
            }
            return parts;
        }
        if state.can_be_scoped() {
            if let Some(s) = &self.scope {
                parts.push(s);
            }
        }
        if state.auto_refresh.is_paused() {
            if let Some(s) = &self.unpause {
                parts.push(s);
            }
        }
        if state.cmd_result.suggest_backtrace() {
            if let Some(s) = &self.toggle_backtrace {
                parts.push(s);
            }
        }
        if let Some(s) = &self.search {
            parts.push(s);
        }
        if let CommandResult::Report(report) = &state.cmd_result {
            if !state.mission.is_success(report) {
                if let Some(s) = &self.toggle_summary {
                    parts.push(s);
                }
            }
        }
        if let Some(s) = &self.help {
            parts.push(s);
        }
        if state.wrap {
            if let Some(s) = &self.not_wrap {
                parts.push(s);
            }
        } else {
            if let Some(s) = &self.wrap {
                parts.push(s);
            }
        }
        if state.auto_refresh.is_enabled() {
            if let Some(s) = &self.pause {
                parts.push(s);
            }
        }
        parts.push(&self.quit);
        parts
    }
    pub fn markdown(
        &self,
        state: &AppState,
    ) -> String {
        let parts = self.applicable_parts(state);
        format!("Hit {}", parts.join(", "))
    }
}
