use {
    crate::*,
    crossterm::{
        self, cursor,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    std::{
        env,
        fs,
        io::Write,
    },
};

/// the type used by all GUI writing functions
///
/// Right now we use stderr, which has the advantage of letting
/// us output something if we want (for a calling process) but
/// as I'm not sure I'll even have something to output, I may
/// switch to stdout which would allow buffering.
pub type W = std::io::Stderr;
//pub type W = std::io::BufWriter<std::io::Stdout>;

/// return the writer used by the application
pub fn writer() -> W {
    std::io::stderr()
    //std::io::BufWriter::new(std::io::stdout())
}

pub fn run() -> Result<()> {
    let mut args: Args = argh::from_env();
    args.fix()?;
    if args.version {
        println!(
            "bac\u{1b}[38;5;204mo\u{1b}[0mn {}",
            env!("CARGO_PKG_VERSION"),
        );
        return Ok(());
    }
    info!("args: {:#?}", &args);
    let location = MissionLocation::new(&args)?;
    debug!("cargo_toml_file: {:?}", &location.cargo_toml_file);

    let package_config_path = location.package_config_path();
    if args.init {
        if package_config_path.exists() {
            eprintln!(
                "{:?} already exists.\nPlease remove it before running bacon --init",
                &package_config_path,
            );
        } else {
            fs::write(&package_config_path, DEFAULT_PACKAGE_CONFIG)?;
            println!(
                "{:?} written.\nYou can modify it.",
                &package_config_path,
            );
        }
        return Ok(())
    }

    let package_config = if package_config_path.exists() {
        PackageConfig::from_path(&package_config_path)?
    } else {
        PackageConfig::default()
    };

    let display_settings = DisplaySettings::from(&args);
    let mission = Mission::new(
        location,
        &package_config,
        args.job.as_deref(),
        display_settings,
    )?;
    info!("mission: {:#?}", &mission);
    let mut w = writer();
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    let r = app::run(&mut w, mission);
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    r
}
