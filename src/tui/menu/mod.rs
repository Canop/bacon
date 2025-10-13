mod action_menu;
mod inform;
mod menu_state;
mod menu_view;

pub use {
    action_menu::*,
    inform::*,
    menu_state::*,
    menu_view::*,
};

use {
    crate::*,
    anyhow::Result,
    crokey::KeyCombination,
    termimad::Area,
};

pub struct Menu<I> {
    pub state: MenuState<I>,
    view: MenuView,
}

impl<I: Md + Clone> Menu<I> {
    pub fn new() -> Self {
        Self {
            state: Default::default(),
            view: Default::default(),
        }
    }
    pub fn draw(
        &mut self,
        w: &mut W,
        skin: &BaconSkin,
    ) -> Result<()> {
        self.view.draw(w, &mut self.state, skin)
    }
    pub fn set_available_area(
        &mut self,
        area: Area,
    ) {
        self.view.set_available_area(area);
    }
    pub fn set_intro<S: Into<String>>(
        &mut self,
        intro: S,
    ) {
        self.state.set_intro(intro);
    }
    pub fn add_item(
        &mut self,
        action: I,
        key: Option<KeyCombination>,
    ) {
        self.state.add_item(action, key);
    }
}
