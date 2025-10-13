use {
    crate::Md,
    crokey::{
        KeyCombination,
        crossterm::event::{
            MouseButton,
            MouseEvent,
            MouseEventKind,
        },
        key,
    },
    termimad::Area,
};

pub struct MenuItem<I> {
    pub action: I,
    pub area: Option<Area>,
    pub key: Option<KeyCombination>,
}

pub struct MenuState<I> {
    pub intro: Option<String>,
    pub items: Vec<MenuItem<I>>,
    pub selection: usize,
    pub scroll: usize,
}

impl<I> Default for MenuState<I> {
    fn default() -> Self {
        Self {
            intro: None,
            items: Vec::new(),
            selection: 0,
            scroll: 0,
        }
    }
}

impl<I: Md + Clone> MenuState<I> {
    pub fn set_intro<S: Into<String>>(
        &mut self,
        intro: S,
    ) {
        self.intro = Some(intro.into());
    }
    pub fn add_item(
        &mut self,
        action: I,
        key: Option<KeyCombination>,
    ) {
        self.items.push(MenuItem {
            action,
            area: None,
            key,
        });
    }
    pub fn clear_item_areas(&mut self) {
        for item in self.items.iter_mut() {
            item.area = None;
        }
    }
    pub fn select(
        &mut self,
        selection: usize,
    ) {
        self.selection = selection.min(self.items.len());
    }
    pub(crate) fn fix_scroll(
        &mut self,
        page_height: usize,
    ) {
        let len = self.items.len();
        let sel = self.selection;
        if len <= page_height || sel < 3 || sel <= page_height / 2 {
            self.scroll = 0;
        } else if sel + 3 >= len {
            self.scroll = len - page_height;
        } else {
            self.scroll = (sel - 2).min(len - page_height);
        }
    }
    /// Handle a key event (not triggering the actions on their keys, only apply
    /// the menu mechanics).
    ///
    /// Return an optional action and a bool telling whether the event was
    ///  consumed by the menu.
    pub fn on_key(
        &mut self,
        key: KeyCombination,
    ) -> (Option<I>, bool) {
        let items = &self.items;
        if key == key!(down) {
            self.selection = (self.selection + 1) % items.len();
            return (None, true);
        } else if key == key!(up) {
            self.selection = (self.selection + items.len() - 1) % items.len();
            return (None, true);
        } else if key == key!(enter) {
            return (Some(items[self.selection].action.clone()), true);
        }
        for item in &self.items {
            if item.key == Some(key) {
                return (Some(item.action.clone()), true);
            }
        }
        (None, false)
    }
    pub fn item_idx_at(
        &self,
        x: u16,
        y: u16,
    ) -> Option<usize> {
        for (idx, item) in self.items.iter().enumerate() {
            if let Some(area) = &item.area {
                if area.contains(x, y) {
                    return Some(idx);
                }
            }
        }
        None
    }
    /// handle a mouse event, returning the triggered action if any (on
    /// double click only)
    pub fn on_mouse_event(
        &mut self,
        mouse_event: MouseEvent,
        double_click: bool,
    ) -> Option<I> {
        let is_click = matches!(
            mouse_event.kind,
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left),
        );
        if is_click {
            if let Some(selection) = self.item_idx_at(mouse_event.column, mouse_event.row) {
                self.selection = selection;
                if double_click {
                    return Some(self.items[self.selection].action.clone());
                }
            }
        }
        None
    }
}
