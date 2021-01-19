use {
    crate::*,
    crossterm::{
        self, cursor,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
    },
    directories_next::ProjectDirs,
    std::{env, fs, io::Write},
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

pub fn run() -> anyhow::Result<()> {
    let mut args: Args = argh::from_env();
    args.fix()?;
    if args.version {
        println!(
            //"bac\u{1b}[38;5;204mo\u{1b}[0mn {}",
            "bac\u{1b}[1m\u{1b}[38;5;204m⚉\u{1b}[0mn {}",
            env!("CARGO_PKG_VERSION"),
        );
        return Ok(());
    }
    info!("args: {:#?}", &args);
    let location = MissionLocation::new(&args)?;
    debug!("cargo_toml_file: {:?}", &location.cargo_toml_file);

    let mut settings = Settings::default();

    if let Some(project_dir) = ProjectDirs::from("org", "dystroy", "bacon") {
        let prefs_path = project_dir.config_dir().join("prefs.toml");
        if args.prefs {
            if !prefs_path.exists() {
                fs::create_dir_all(prefs_path.parent().unwrap())?;
                fs::write(&prefs_path, DEFAULT_PREFS)?;
                // written to stderr to allow initialization with commands like
                //  $EDITOR "$(bacon --prefs)"
                eprintln!("Preferences file written.");
            }
            println!("{}", prefs_path.to_string_lossy());
            return Ok(());
        }
        if prefs_path.exists() {
            let prefs = Prefs::from_path(&prefs_path)?;
            info!("prefs: {:#?}", &prefs);
            settings.apply_prefs(&prefs);
        }
    }

    let package_config_path = location.package_config_path();
    if args.init {
        if !package_config_path.exists() {
            fs::write(&package_config_path, DEFAULT_PACKAGE_CONFIG)?;
            eprintln!("bacon project configuration file written.");
        } else {
            eprintln!("bacon configuration file already exists.");
        }
        println!("{}", package_config_path.to_string_lossy());
        return Ok(());
    }
    let package_config = if package_config_path.exists() {
        PackageConfig::from_path(&package_config_path)?
    } else {
        PackageConfig::default()
    };

    // args are applied after prefs, so that they can override them
    settings.apply_args(&args);

    let mission = Mission::new(location, &package_config, args.job.as_deref(), settings)?;
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
