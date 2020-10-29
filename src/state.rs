use {
    crate::*,
    anyhow::*,
    crossbeam::channel::{
        Receiver,
        Sender,
        select,
        unbounded,
    },
    crossterm::{
        cursor,
        execute,
        ExecutableCommand,
        event::KeyModifiers,
        QueueableCommand,
        style::{Colorize, Styler},
        terminal,
    },
    std::{
        env,
        io::Write,
        path::PathBuf,
    },
    termimad::{Event, EventSource},
};

#[derive(Debug)]
pub struct AppState {
    pub name: String,
    pub report: Option<Report>,
    pub screen: (u16, u16),
    pub computing: bool,
    pub summary: bool,
}
impl AppState {
    pub fn new() -> Result<Self> {
        Ok(Self {
            name: env::current_dir()?.file_name().unwrap().to_string_lossy().to_string(),
            report: None,
            screen: termimad::terminal_size(),
            computing: true,
            summary: false,
        })
    }
}

fn goto(w: &mut W, y: u16) -> Result<()> {
    execute!(w, cursor::MoveTo(0, y), terminal::Clear(terminal::ClearType::CurrentLine))?;
    Ok(())
}

impl AppState {
    pub fn draw(
        &self,
        w: &mut W,
    ) -> Result<()> {
        let width = self.screen.0 as usize;
        goto(w, 1)?;
        if self.computing {
            // todo: maybe show the current line of the computed report ?
            eprint!("{}",
                format!("{:^w$}", "computing...", w=width).white().on_dark_grey()
            );
        }
        if let Some(report) = &self.report {
            goto(w, 0)?;
            eprint!("{} ", format!(" {} ", &self.name).white().bold().on_dark_grey());
            if !report.warnings.is_empty() {
                eprint!("{} ", format!(" {} warnings ", report.warnings.len()).black().bold().on_yellow());
            }
            if !report.errors.is_empty() {
                eprint!("{} ", format!(" {} errors ", report.errors.len()).white().bold().on_red());
            }
            if report.warnings.is_empty() && report.errors.is_empty() {
                eprint!("{} ", " pass! ".white().bold().on_dark_green());
            }
            let mut y = 2;
            let mut w_idx = 0;
            let mut e_idx = 0;
            let h = self.screen.1 - 1;
            while y < h {
                goto(w, y)?;
                if let Some(ref error) = report.errors.get(e_idx) {
                    eprint!(
                        "{} {}",
                        format!("{:>2} ", e_idx + 1).black().bold().on_red(),
                        &error.lines[0],
                    );
                    if !self.summary {
                        for line in &error.lines[1..] {
                            if y >= h - 1 {
                                break;
                            }
                            y += 1;
                            goto(w, y)?;
                            eprint!(" {}", &line);
                        }
                    }
                    e_idx += 1;
                } else if let Some(ref warning) = report.warnings.get(w_idx) {
                    eprint!(
                        "{} {}",
                        format!("{:>2} ", w_idx + 1).black().bold().on_yellow(),
                        &warning.lines[0],
                    );
                    if !self.summary {
                        for line in &warning.lines[1..] {
                            if y >= h - 1 {
                                break;
                            }
                            y += 1;
                            goto(w, y)?;
                            eprint!(" {}", &line);
                        }
                    }
                    w_idx += 1;
                }
                y += 1;
            }
        }
        goto(w, self.screen.1)?;
        eprint!("{}",
            format!(" {:<w$}", "hit q to quit", w=width-1).white().on_dark_grey()
        );
        Ok(())
    }
}

