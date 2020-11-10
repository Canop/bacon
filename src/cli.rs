use {
    crate::*,
    argh::FromArgs,
    crossterm::{
        self, cursor,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    std::{env, io::Write, path::PathBuf},
};

#[derive(Debug, FromArgs)]
/// bacon watches your source and run code checks in background.
///
///
/// Source at https://github.com/Canop/bacon
pub struct Args {
    /// print the version
    #[argh(switch, short = 'v')]
    version: bool,

    /// whether to start in summary mode
    #[argh(switch, short = 's')]
    pub summary: bool,

    /// whether to start with lines wrapped
    #[argh(switch, short = 'w')]
    pub wrap: bool,

    /// whether to run `cargo clippy` instead of `cargo check`
    #[argh(switch, short = 'c')]
    pub clippy: bool,

    #[argh(positional)]
    /// path to the folder to watch or to the Rust repository
    pub root: Option<PathBuf>,
}

/// the type used by all GUI writing functions
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
