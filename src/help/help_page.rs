use {
    crate::*,
    anyhow::Result,
    termimad::{
        Area,
        CompoundStyle,
        FmtText,
        MadSkin,
        TextView,
        crossterm::style::Attribute,
        minimad::{
            Alignment,
            OwningTemplateExpander,
            TextTemplate,
        },
    },
};

static TEMPLATE: &str = r"

# bacon ${version}

**bac*o*n** is a background compiler, watching your sources and executing your cargo jobs on change.

See *https://dystroy.org/bacon* for a complete guide.

|:-:|:-:
|**action**|**shortcut**
|:-|:-:
${keybindings
|${action}|${keys}
}
|-:

Key bindings, jobs, and other preferences were loaded from these files:
* internal default configuration files
${config_files
* ${config_file_path}
}


";

pub struct HelpPage {
    area: Area,
    skin: MadSkin,
    expander: OwningTemplateExpander<'static>,
    template: TextTemplate<'static>,
    scroll: usize,
}

impl HelpPage {
    pub fn new(settings: &Settings) -> Self {
        let mut skin = MadSkin::default();
        skin.paragraph.align = Alignment::Center;
        let key_color = settings.all_jobs.skin.key_fg.color();
        skin.italic = CompoundStyle::new(Some(key_color), None, Attribute::Bold.into());
        skin.table.align = Alignment::Center;
        skin.bullet.set_fg(key_color);
        let mut expander = OwningTemplateExpander::new();
        expander.set("version", env!("CARGO_PKG_VERSION"));
        let mut bindings: Vec<(String, String)> = settings
            .keybindings
            .build_reverse_map()
            .into_iter()
            .map(|(action, cks)| {
                let cks: Vec<String> = cks.iter().map(|ck| format!("*{ck}*")).collect();
                let cks = cks.join(" or ");
                (action.md(), cks)
            })
            .collect();
        bindings.sort_by(|a, b| a.0.cmp(&b.0));
        for (action, key) in bindings.drain(..) {
            expander
                .sub("keybindings")
                .set_md("keys", key)
                .set_md("action", action);
        }
        for config_file in &settings.config_files {
            expander
                .sub("config_files")
                .set_md("config_file_path", config_file.to_string_lossy());
        }
        let template = TextTemplate::from(TEMPLATE);
        Self {
            area: Area::default(),
            skin,
            expander,
            template,
            scroll: 0,
        }
    }

    /// draw the state on the whole terminal
    pub fn draw(
        &mut self,
        w: &mut W,
        area: Area,
    ) -> Result<()> {
        self.area = area;
        let text = self.expander.expand(&self.template);
        let fmt_text = FmtText::from_text(&self.skin, text, Some((self.area.width - 1) as usize));
        let mut text_view = TextView::from(&self.area, &fmt_text);
        self.scroll = text_view.set_scroll(self.scroll);
        Ok(text_view.write_on(w)?)
    }

    pub fn apply_scroll_command(
        &mut self,
        cmd: ScrollCommand,
    ) {
        let text = self.expander.expand(&self.template);
        let fmt_text = FmtText::from_text(&self.skin, text, Some((self.area.width - 1) as usize));
        let mut text_view = TextView::from(&self.area, &fmt_text);
        text_view.set_scroll(self.scroll);
        match cmd {
            ScrollCommand::Top => {
                text_view.scroll = 0;
            }
            ScrollCommand::Bottom => {
                text_view.set_scroll(text_view.content_height());
            }
            ScrollCommand::Lines(lines) => {
                text_view.try_scroll_lines(lines);
            }
            ScrollCommand::MilliPages(milli_pages) => {
                text_view.try_scroll_pages(f64::from(milli_pages) / 1000f64);
            }
        }
        self.scroll = text_view.scroll;
    }
}
