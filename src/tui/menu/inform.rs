use super::*;

static OK: &str = "OK";

pub fn inform<S: Into<String>>(txt: S) -> Menu<&'static str> {
    let mut menu = Menu::new();
    menu.add_item(OK, None);
    menu.state.set_intro(txt);
    menu
}

pub type InformMenu = Menu<&'static str>;
