use crate::*;

pub struct HelpLine {
    clear_search: Option<String>,
    close_help: Option<String>,
    help: Option<String>,
    next_match: Option<String>,
    not_wrap: Option<String>,
    pause: Option<String>,
    previous_match: Option<String>,
    quit: String,
    scope: Option<String>,
    search: Option<String>,
    toggle_backtrace: Option<String>,
    toggle_summary: Option<String>,
    undismiss: Option<String>,
    unpause: Option<String>,
    validate_search: Option<String>,
    wrap: Option<String>,
}

impl HelpLine {
    /// Create a new HelpLine based on the provided settings.
    ///
    /// # Panics
    /// Panics if there is no keybinding for quitting the application.
    /// But that's good: it's better to die if the user can't kill the app...
    pub fn new(settings: &Settings) -> Self {
        let kb = &settings.keybindings;
        let quit = kb
            .shortest_key_for(&Action::Quit)
            .map(|k| format!("*{k}* to quit"))
            .expect("the app to be quittable");
        let toggle_summary = kb
            .shortest_key_for(&Action::ToggleSummary)
            .map(|k| format!("*{k}* to toggle summary mode"));
        let wrap = kb
            .shortest_key_for(&Action::ToggleWrap)
            .map(|k| format!("*{k}* to wrap lines"));
        let not_wrap = kb
            .shortest_key_for(&Action::ToggleWrap)
            .map(|k| format!("*{k}* to not wrap lines"));
        let toggle_backtrace = kb
            .shortest_key(|action| matches!(action, Action::ToggleBacktrace(_)))
            .map(|k| format!("*{k}* to toggle backtraces"));
        let help = kb
            .shortest_key_for(&Action::Help)
            .map(|k| format!("*{k}* for help"));
        let close_help = kb
            .shortest_key_for(&Action::Back)
            .or_else(|| kb.shortest_key_for(&Action::Help))
            .map(|k| format!("*{k}* to close this help"));
        let pause = kb
            .shortest_key_for(&Action::Pause)
            .or(kb.shortest_key_for(&Action::TogglePause))
            .map(|k| format!("*{k}* to pause"));
        let unpause = kb
            .shortest_key_for(&Action::Unpause)
            .or(kb.shortest_key_for(&Action::TogglePause))
            .map(|k| format!("*{k}* to unpause"));
        let scope = kb
            .shortest_key_for(&Action::ScopeToFailures)
            .map(|k| format!("*{k}* to scope to failures"));
        let search = kb
            .shortest_key_for(&Action::FocusSearch)
            .map(|k| format!("*{k}* to search"));
        let next_match = kb
            .shortest_key_for(&Action::NextMatch)
            .map(|k| format!("*{k}* for next match"));
        let previous_match = kb
            .shortest_key_for(&Action::PreviousMatch)
            .map(|k| format!("*{k}* for previous match"));
        let clear_search = kb
            .shortest_key_for(&Action::Back)
            .map(|k| format!("*{k}* to clear"));
        let validate_search = kb
            .shortest_key_for(&Action::Validate)
            .map(|k| format!("*{k}* to validate"));
        let undismiss = kb
            .shortest_key_for(&Action::OpenUndismissMenu)
            .map(|k| format!("*{k}* to undismiss"));
        Self {
            clear_search,
            close_help,
            help,
            next_match,
            not_wrap,
            pause,
            previous_match,
            quit,
            scope,
            search,
            toggle_backtrace,
            toggle_summary,
            undismiss,
            unpause,
            validate_search,
            wrap,
        }
    }
    fn applicable_parts(
        &self,
        state: &MissionState,
    ) -> Vec<&str> {
        let mut parts: Vec<&str> = Vec::new();
        if state.is_help() {
            parts.push(&self.quit);
            if let Some(s) = &self.close_help {
                parts.push(s);
            }
            return parts;
        }
        if state.has_dismissed_items() {
            if let Some(s) = &self.undismiss {
                parts.push(s);
            }
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
        state: &MissionState,
    ) -> String {
        let parts = self.applicable_parts(state);
        format!("Hit {}", parts.join(", "))
    }
}
