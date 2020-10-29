use {
    crate::*,
    crossterm::{
        self,
        cursor,
        event::{DisableMouseCapture, EnableMouseCapture},
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    std::{
        env,
        io::{self, Write},
        path::{Path, PathBuf},
        str::FromStr,
    },
};

/// the type used by all GUI writing functions
//pub type W = std::io::BufWriter<std::io::Stderr>;
pub type W = std::io::Stderr;

/// return the writer used by the application
pub fn writer() -> W {
    //std::io::BufWriter::new(std::io::stderr())
    std::io::stderr()
}

pub fn run() -> Result<()> {
    let mut w = writer();
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::DisableBlinking)?;
    w.queue(cursor::Hide)?;
    // if !config.disable_mouse_capture {
    //     w.queue(EnableMouseCapture)?;
    // }
    let r = app::run(&mut w);
    // if !config.disable_mouse_capture {
    //     w.queue(DisableMouseCapture)?;
    // }
    w.queue(cursor::Show)?;
    w.queue(cursor::EnableBlinking)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    r
}
