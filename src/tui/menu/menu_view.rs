use {
    crate::*,
    anyhow::Result,
    crokey::{
        KeyCombinationFormat,
        crossterm::{
            queue,
            style::Print,
        },
    },
    termimad::{
        minimad::*,
        *,
    },
};

/// The drawer of the menu
#[derive(Default)]
pub struct MenuView {
    available_area: Area,
}

impl MenuView {
    fn compute_area_width(
        &self,
        content_width: u16,
    ) -> u16 {
        let screen = &self.available_area;
        let sw2 = screen.width / 2;
        let w2 = 24.max(content_width / 2 + 1).min(sw2 - 3); // menu half width
        w2 * 2
    }
    fn compute_area(
        &self,
        content_height: usize,
        area_width: u16,
    ) -> Area {
        let screen = &self.available_area;
        let ideal_height = content_height as u16 + 2; // margin of 1
        let left = (screen.width - area_width) / 2;
        let h = screen.height.min(ideal_height);
        let top = ((screen.height - h) * 3 / 5).max(1);
        Area::new(left, top, area_width, h)
    }
    pub fn set_available_area(
        &mut self,
        available_area: Area,
    ) {
        if available_area != self.available_area {
            self.available_area = available_area;
        }
    }

    fn estimate_content_optimal_width<I: Md>(state: &MenuState<I>) -> u16 {
        state
            .items
            .iter()
            .map(|item| item.action.md().len() + 8)
            .max()
            .unwrap_or(0)
            .try_into()
            .unwrap_or(100)
    }

    /// Draw the menu and set the area of all visible items in the state
    pub fn draw<I: Md + Clone>(
        &mut self,
        w: &mut W,
        state: &mut MenuState<I>,
        skin: &BaconSkin,
    ) -> Result<()> {
        let key_format = KeyCombinationFormat::default().with_implicit_shift();
        //.with_control("^");
        state.clear_item_areas();
        let mut md_skin = MadSkin::default();
        md_skin
            .paragraph
            .set_fgbg(skin.menu_item_fg.color(), skin.menu_item_bg.color());
        md_skin.italic.set_fg(skin.key_fg.color());
        let mut sel_md_skin = MadSkin::default();
        sel_md_skin.paragraph.set_fgbg(
            skin.menu_item_selected_fg.color(),
            skin.menu_item_selected_bg.color(),
        );
        sel_md_skin.italic.set_fg(skin.key_fg.color());
        let area_width = self.compute_area_width(Self::estimate_content_optimal_width(state));
        let mut intro_lines = Vec::new();
        let text_width = area_width - 2;
        let intro = state.intro.clone();
        let mut content_height = state.items.len();
        if let Some(intro) = &intro {
            let text = FmtText::from(&md_skin, intro, Some(text_width as usize));
            intro_lines = text.lines;
            content_height += intro_lines.len() + 1; // 1 for margin
        }
        let area = self.compute_area(content_height, area_width);
        let h = area.height as usize - 2; // internal height
        let scrollbar = compute_scrollbar(state.scroll, content_height, h, area.top + 1);
        state.fix_scroll(h);
        let mut rect = Rect::new(
            area.clone(),
            CompoundStyle::with_fgbg(skin.menu_border.color(), skin.menu_bg.color()),
        );
        rect.set_border_style(BORDER_STYLE_HALF_WIDTH_OUTSIDE);
        rect.set_fill(true);
        rect.draw(w)?;
        let key_width = 8;
        let mut label_width = area.width as usize - key_width - 2;
        if scrollbar.is_some() {
            label_width -= 1;
        }
        let mut y = area.top;
        let mut items = state.items.iter_mut().enumerate().skip(state.scroll);
        for _ in 0..h {
            y += 1;
            if !intro_lines.is_empty() {
                let intro_line = intro_lines.remove(0);
                goto(w, area.left + 1, y)?;
                let dl = DisplayableLine::new(&md_skin, &intro_line, Some(text_width as usize));
                queue!(w, Print(&dl))?;
                if intro_lines.is_empty() {
                    y += 1; // skip line for margin
                }
                continue;
            }
            if let Some((idx, item)) = items.next() {
                let item_area = Area::new(area.left + 1, y, area.width - 2, 1);
                let skin = if state.selection == idx {
                    &sel_md_skin
                } else {
                    &md_skin
                };
                goto(w, item_area.left, y)?;
                skin.write_composite_fill(
                    w,
                    Composite::from_inline(&item.action.md()),
                    label_width,
                    Alignment::Left,
                )?;
                let key_desc = item
                    .key
                    .map_or(String::new(), |key| key_format.to_string(key));
                skin.write_composite_fill(
                    w,
                    mad_inline!("*$0", &key_desc),
                    key_width,
                    Alignment::Right,
                )?;
                item.area = Some(item_area);
            } else {
                break;
            }
            if let Some((stop, sbottom)) = scrollbar {
                goto(w, area.right() - 2, y)?;
                if stop <= y && y <= sbottom {
                    md_skin.scrollbar.thumb.queue(w)?;
                } else {
                    md_skin.scrollbar.track.queue(w)?;
                }
            }
        }
        Ok(())
    }
}
