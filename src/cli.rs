use {
    crate::*,
    crossterm::{
        self, cursor,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    directories_next::ProjectDirs,
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
//pub type W = std::io::Stderr;
pub type W = std::io::BufWriter<std::io::Stdout>;

/// return the writer used by the application
pub fn writer() -> W {
    //std::io::stderr()
    std::io::BufWriter::new(std::io::stdout())
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

    let mut display_settings = DisplaySettings::default();

    if let Some(project_dir) = ProjectDirs::from("org", "dystroy", "bacon") {
        let prefs_path = project_dir.config_dir().join("prefs.toml");
        if args.prefs {
            if prefs_path.exists() {
                println!("bacon prefs are at\n{:?}", &prefs_path);
            } else {
                fs::create_dir_all(prefs_path.parent().unwrap())?;
                fs::write(&prefs_path, DEFAULT_PREFS)?;
                println!(
                    "{:?} file written.\nYou can modify it.",
                    &prefs_path,
                );
            }
            return Ok(());
        }
        if prefs_path.exists() {
            let prefs = Prefs::from_path(&prefs_path)?;
            info!("prefs: {:#?}", &prefs);
            display_settings.apply_prefs(&prefs);
        }
    }

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
                "{:?} file written.\nYou can modify it.",
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

    // args are applied after prefs, so that they can override them
    display_settings.apply_args(&args);

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
