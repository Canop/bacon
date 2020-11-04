use {
    crate::*,
    anyhow::*,
    crossterm::{
        cursor, execute,
        style::{Color::*, Colorize, Styler},
        terminal,
    },
    minimad::{Alignment, Composite},
    std::{io::Write, path::Path},
    termimad::{CompoundStyle, MadSkin},
};

pub struct AppState {
    pub name: String,
    pub report: Option<Report>,
    pub screen: (u16, u16),
    pub computing: bool,
    pub summary: bool,
    pub status_skin: MadSkin,
}
impl AppState {
    pub fn new(root_dir: &Path) -> Result<Self> {
        let mut status_skin = MadSkin::default();
        status_skin.paragraph.set_bg(DarkGrey);
        status_skin.italic = CompoundStyle::with_fg(Yellow);
        Ok(Self {
            name: root_dir.file_name().unwrap().to_string_lossy().to_string(),
            report: None,
            screen: termimad::terminal_size(),
            computing: true,
            summary: false,
            status_skin,
        })
    }
}

fn goto(w: &mut W, y: u16) -> Result<()> {
    execute!(
        w,
        cursor::MoveTo(0, y),
        terminal::Clear(terminal::ClearType::CurrentLine)
    )?;
    Ok(())
}

impl AppState {
    pub fn draw(&self, w: &mut W) -> Result<()> {
        let width = self.screen.0 as usize;
        goto(w, 1)?;
        if self.computing {
            // todo: maybe show the current line of the computed report ?
            eprint!(
                "{}",
                format!("{:^w$}", "computing...", w = width)
                    .white()
                    .on_dark_grey()
            );
        }
        if let Some(report) = &self.report {
            goto(w, 0)?;
            eprint!(
                "{} ",
                format!(" {} ", &self.name).white().bold().on_dark_grey()
            );
            let w_count = report.warnings.len();
            let e_count = report.errors.len();
            if w_count > 0 {
                let s = if w_count == 1 {
                    " 1 warning ".to_string()
                } else {
                    format!(" {} warnings ", w_count)
                };
                eprint!("{} ", s.black().bold().on_yellow());
            }
            if e_count > 0 {
                let s = if e_count == 1 {
                    " 1 error ".to_string()
                } else {
                    format!(" {} errors ", e_count)
                };
                eprint!("{} ", s.black().bold().on_red());
            }
            if w_count == 0 && e_count == 0 {
                eprint!("{} ", " pass! ".white().bold().on_dark_green());
            }
            let mut y = 2;
            let h = self.screen.1 - 1;
            let mut idx = 1;
            let mut errors = report.errors.iter();
            let mut warnings = report.warnings.iter();
            while y < h {
                goto(w, y)?;
                if let Some(ref error) = errors.next() {
                    eprint!(
                        "{} {}",
                        format!("{:>2} ", idx).black().bold().on_red(),
                        &error.lines[0].content,
                    );
                    for line in &error.lines[1..error.lines.len()] {
                        if self.summary && !line.summary {
                            continue;
                        }
                        if y >= h - 1 {
                            break;
                        }
                        y += 1;
                        goto(w, y)?;
                        eprint!(" {}", &line.content);
                    }
                    idx += 1;
                } else if let Some(ref warning) = warnings.next() {
                    eprint!(
                        "{} {}",
                        format!("{:>2} ", idx).black().bold().on_yellow(),
                        &warning.lines[0].content,
                    );
                    for line in &warning.lines[1..warning.lines.len()] {
                        if self.summary && !line.summary {
                            continue;
                        }
                        if y >= h - 1 {
                            break;
                        }
                        y += 1;
                        goto(w, y)?;
                        eprint!(" {}", &line.content);
                    }
                    idx += 1;
                }
                y += 1;
            }
        }
        goto(w, self.screen.1)?;
        let status = "hit *q* to quit, *s* to toggle summary mode";
        self.status_skin.write_composite_fill(
            w,
            Composite::from_inline(status),
            width,
            Alignment::Left,
        )?;
        Ok(())
    }
}
