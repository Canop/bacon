use {
    crate::*,
    clap::Parser,
    directories_next::ProjectDirs,
    std::{fs, io::Write},
    termimad::EventSource,
    termimad::crossterm::{
        cursor,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
        QueueableCommand,
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

pub fn run() -> anyhow::Result<()> {
    let mut args: Args = Args::parse();
    args.fix()?;
    info!("args: {:#?}", &args);

    let mut settings = Settings::default();

    if let Some(project_dir) = ProjectDirs::from("org", "dystroy", "bacon") {
        let prefs_path = project_dir.config_dir().join("prefs.toml");
        if args.prefs {
            if !prefs_path.exists() {
                fs::create_dir_all(prefs_path.parent().unwrap())?;
                fs::write(&prefs_path, DEFAULT_PREFS.trim_start())?;
                // written to stderr to allow initialization with commands like
                //  $EDITOR "$(bacon --prefs)"
                eprintln!("Preferences file written.");
            }
            println!("{}", prefs_path.to_string_lossy());
            return Ok(());
        }
        if prefs_path.exists() {
            let prefs = Config::from_path(&prefs_path)?;
            debug!("prefs: {:#?}", &prefs);
            settings.apply_config(&prefs);
        }
    }

    let location = MissionLocation::new(&args)?;
    info!("mission location: {:#?}", &location);

    let package_config_path = location.package_config_path();
    if args.init {
        if !package_config_path.exists() {
            fs::write(&package_config_path, DEFAULT_PACKAGE_CONFIG.trim_start())?;
            eprintln!("bacon project configuration file written.");
        } else {
            eprintln!("bacon configuration file already exists.");
        }
        println!("{}", package_config_path.to_string_lossy());
        return Ok(());
    }
    let package_config = if package_config_path.exists() {
        Config::from_path(&package_config_path)?
    } else {
        Config::default_package_config()
    };
    settings.apply_config(&package_config);

    // args are applied after prefs, and package config so that they can override them
    settings.apply_args(&args);

    settings.check()?;

    if args.list_jobs {
        print_jobs(&settings);
        return Ok(());
    }

    let mut w = writer();
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;

    let event_source = EventSource::new()?;
    let mut job_stack = JobStack::new(&settings);
    let mut next_job = JobRef::Initial;
    let mut result = Ok(());
    #[allow(clippy::while_let_loop)]
    loop {
        let (job_name, job) = match job_stack.pick_job(&next_job) {
            Err(e) => {
                result = Err(e);
                break;
            }
            Ok(Some(t)) => t,
            Ok(None) => { break; }
        };
        let r = Mission::new(&location, job_name.to_string(), job, &settings)
            .and_then(|mission| app::run(&mut w, mission, &event_source));
        match r {
            Ok(Some(job_ref)) => {
                next_job = job_ref;
            }
            Ok(None) => {
                break;
            }
            Err(e) => {
                result = Err(e);
                break;
            }
        }
    }

    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    result
}
