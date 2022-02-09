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
            "bacon {}",
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
                fs::write(&prefs_path, DEFAULT_PREFS.trim_start())?;
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
            fs::write(&package_config_path, DEFAULT_PACKAGE_CONFIG.trim_start())?;
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
    settings.apply_package_config(&package_config);

    // args are applied after prefs, and package config so that they can override them
    settings.apply_args(&args);

    if args.list_jobs {
        print_jobs(&package_config);
        return Ok(());
    }

    let mut next_job: Option<JobRef> = Some(JobRef::Initial);
    let mut w = writer();
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    let mut result = Ok(());
    while let Some(job_ref) = next_job.take() {
        let r = Mission::new(&location, &package_config, job_ref, &settings)
            .and_then(|mission| app::run(&mut w, mission));
        match r {
            Ok(returned_next_job) => {
                next_job = returned_next_job;
            }
            Err(e) => {
                result = Err(e);
            }
        }
    }
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    result
}
