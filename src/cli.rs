use {
    crate::*,
    crossterm::{
        self, cursor,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    std::{env, io::Write},
};

/// the type used by all GUI writing functions
///
/// Right now we use stderr, which has the advantage of letting
/// us output something if we want (for a calling process) but
/// as I'm not sure I'll even have something to output, I may
/// switch to stdout which would allow buffering.
pub type W = std::io::Stderr;

/// return the writer used by the application
pub fn writer() -> W {
    //std::io::BufWriter::new(std::io::stderr())
    std::io::stderr()
}

pub fn run() -> Result<()> {
    let args: Args = argh::from_env();
    if args.version {
        println!(
            "bac\u{1b}[38;5;204mo\u{1b}[0mn {}",
            //"bac\u{1b}[38;5;204m\u{25cf}\u{1b}[0mn {}",
            env!("CARGO_PKG_VERSION"),
        );
        return Ok(());
    }
    debug!("args: {:#?}", &args);
    let mission = Mission::from(args)?;
    debug!("mission: {:#?}", &mission);
    let mut w = writer();
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    let r = app::run(&mut w, mission);
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    r
}
