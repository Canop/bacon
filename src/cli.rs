use {
    crate::*,
    clap::Parser,
    directories_next::ProjectDirs,
    std::{
        fs,
        io::Write,
    },
    shlex::try_quote,
    termimad::{
        EventSource,
        EventSourceOptions,
        crossterm::{
            QueueableCommand,
            cursor,
            terminal::{
                EnterAlternateScreen,
                LeaveAlternateScreen,
            },
        },
    },
};

#[cfg(windows)]
use termimad::crossterm::event::{
    DisableMouseCapture,
    EnableMouseCapture,
};
/// The Write type used by all GUI writing functions
pub type W = std::io::BufWriter<std::io::Stdout>;

/// return the writer used by the application
pub fn writer() -> W {
    std::io::BufWriter::new(std::io::stdout())
}

pub fn run() -> anyhow::Result<()> {
    let mut args: Args = Args::parse();
    args.fix()?;
    info!("args: {:#?}", &args);

    if args.help {
        args.print_help();
        return Ok(());
    }

    if args.version {
        println!("bacon {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut settings = Settings::default();

    let default_package_config = Config::default_package_config();
    settings.apply_config(&default_package_config);

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
            println!("{}", try_quote(&prefs_path.to_string_lossy())?);
            return Ok(());
        }
        if prefs_path.exists() {
            let prefs = Config::from_path(&prefs_path)?;
            info!("prefs: {:#?}", &prefs);
            settings.apply_config(&prefs);
        }
    }

    if let Some(config) = Config::from_env("BACON_PREFS")? {
        settings.apply_config(&config);
    }

    let location = MissionLocation::new(&args)?;
    info!("mission location: {:#?}", &location);

    let workspace_config_path = location.workspace_config_path();
    let package_config_path = location.package_config_path();

    if package_config_path != workspace_config_path {
        if workspace_config_path.exists() {
            info!("loading workspace level bacon.toml");
            let workspace_config = Config::from_path(&workspace_config_path)?;
            settings.apply_config(&workspace_config);
        }
    }

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
    if package_config_path.exists() {
        let config = Config::from_path(&package_config_path)?;
        settings.apply_config(&config);
    }

    if let Some(config) = Config::from_env("BACON_CONFIG")? {
        settings.apply_config(&config);
    }

    // args are applied after prefs, and package config so that they can override them
    settings.apply_args(&args);

    settings.check()?;

    info!("settings: {:#?}", &settings);

    if args.list_jobs {
        print_jobs(&settings);
        return Ok(());
    }

    let mut w = writer();
    w.queue(EnterAlternateScreen)?;
    w.queue(cursor::Hide)?;
    #[cfg(windows)]
    w.queue(EnableMouseCapture)?;
    w.flush()?;
    let event_source = EventSource::with_options(EventSourceOptions {
        combine_keys: false,
        ..Default::default()
    })?;
    let mut job_stack = JobStack::new(&settings);
    let mut next_job = JobRef::Initial;
    let mut result = Ok(());
    #[allow(clippy::while_let_loop)]
    loop {
        let (concrete_job_ref, job) = match job_stack.pick_job(&next_job) {
            Err(e) => {
                result = Err(e);
                break;
            }
            Ok(Some(t)) => t,
            Ok(None) => {
                break;
            }
        };
        let r = Mission::new(&location, concrete_job_ref, job, &settings)
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

    #[cfg(windows)]
    w.queue(DisableMouseCapture)?;
    w.queue(cursor::Show)?;
    w.queue(LeaveAlternateScreen)?;
    w.flush()?;
    result
}
