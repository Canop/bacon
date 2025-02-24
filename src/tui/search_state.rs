use {
    crate::*,
    anyhow::Result,
    crokey::KeyCombination,
    termimad::InputField,
};

/// Search related state, part of the app state
pub struct SearchState {
    /// the search input field
    input: InputField,
    /// whether the app state is up to date with the search
    up_to_date: bool,
    /// Locations matching the search_input content
    founds: Vec<Found>,
    /// The selection to show (index among the founds)
    selected_found: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        let mut search_input = InputField::default();
        search_input.set_focus(false);
        let founds = Default::default();
        Self {
            input: search_input,
            up_to_date: true,
            founds,
            selected_found: 0,
        }
    }
}

impl SearchState {
    pub fn is_up_to_date(&self) -> bool {
        self.up_to_date
    }
    /// tell the search state that it's not up to date with the app state
    pub fn touch(&mut self) {
        self.up_to_date = false;
    }
    pub fn selected_found(&self) -> usize {
        self.selected_found
    }
    pub fn has_founds(&self) -> bool {
        !self.founds.is_empty()
    }
    pub fn must_be_drawn(&self) -> bool {
        self.focused() || !self.input.is_empty()
    }
    pub fn set_focus(
        &mut self,
        f: bool,
    ) {
        self.input.set_focus(f);
    }
    pub fn focused(&self) -> bool {
        self.input.focused()
    }
    pub fn input_has_content(&self) -> bool {
        !self.input.is_empty()
    }
    pub fn unfocus_and_clear(&mut self) {
        self.input.clear();
        self.up_to_date = false;
        self.input.set_focus(false);
    }
    pub fn clear(&mut self) {
        self.input.clear();
        self.up_to_date = false;
    }
    pub fn next_match(&mut self) {
        if self.founds.is_empty() {
            return;
        }
        self.selected_found = (self.selected_found + 1) % self.founds.len();
    }
    pub fn previous_match(&mut self) {
        if self.founds.is_empty() {
            return;
        }
        self.selected_found = (self.selected_found + self.founds.len() - 1) % self.founds.len();
    }
    /// handle a raw, uninterpreted key combination (in an input if there's one
    /// focused), return true if the key was consumed (if not, keybindings will
    /// be computed)
    pub fn apply_key_combination(
        &mut self,
        key: KeyCombination,
    ) -> bool {
        if self.input.focused() {
            if self.input.apply_key_combination(key) {
                self.up_to_date = false;
                return true;
            }
        }
        false
    }
    pub fn pattern(&self) -> Pattern {
        Pattern {
            pattern: self.input.get_content(),
        }
    }
    pub fn set_founds(
        &mut self,
        founds: Vec<Found>,
    ) {
        let old_selected_line = self.selected_found_line();
        self.founds = founds;
        let new_selected_line = self.selected_found_line();
        if old_selected_line != new_selected_line {
            self.selected_found = 0;
        }
        self.up_to_date = true;
    }
    pub fn extend_founds(
        &mut self,
        new_founds: Vec<Found>,
    ) {
        self.founds.extend(new_founds);
    }
    /// if there are search results, return the line index of the currently selected one
    pub fn selected_found_line(&self) -> Option<usize> {
        self.founds
            .get(self.selected_found)
            .map(|found| found.line_idx)
    }
    pub fn founds(&self) -> &[Found] {
        &self.founds
    }
    /// Draw at the given position, with the specified width
    pub fn draw_prefixed_input(
        &mut self,
        w: &mut W,
        x: u16,
        y: u16,
        width: u16, // must be > 1
    ) -> Result<()> {
        goto_line(w, y)?;
        draw(w, CSI_FOUND, "/")?;
        self.input.change_area(x + 1, y, width - 1);
        self.input.display_on(w)?;
        Ok(())
    }
    pub fn add_summary_tstring(
        &self,
        t_line: &mut TLine,
    ) {
        if self.input_has_content() {
            if self.founds.is_empty() {
                t_line.add_tstring(CSI_FOUND, "no match");
            } else {
                t_line.add_tstring(
                    CSI_FOUND,
                    format!("{}/{}", self.selected_found + 1, self.founds.len(),),
                );
            }
        }
    }
}
