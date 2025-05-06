mod args;
mod completions;

pub use args::*;

use {
    crate::*,
    anyhow::anyhow,
    clap::{
        CommandFactory,
        Parser,
    },
    std::{
        fs,
        io::Write,
    },
    termimad::crossterm::{
        QueueableCommand,
        cursor,
        terminal::{
            EnterAlternateScreen,
            LeaveAlternateScreen,
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
    if std::env::var_os("COMPLETE").is_some() {
        clap_complete::CompleteEnv::with_factory(Args::command).complete();
    }

    let mut args: Args = Args::parse();
    args.fix()?;
    info!("args: {:#?}", &args);
    let headless = args.headless;

    if args.help {
        args.print_help();
        return Ok(());
    }

    if args.version {
        println!("bacon {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.prefs {
        let prefs_path =
            bacon_prefs_path().ok_or(anyhow!("No preferences location known for this system."))?;
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

    let context = Context::new(&args)?;
    debug!("mission context: {:#?}", &context);

    if args.init {
        let package_config_path = context.package_config_path();
        if !package_config_path.exists() {
            fs::write(&package_config_path, DEFAULT_PACKAGE_CONFIG.trim_start())?;
            eprintln!("bacon project configuration file written.");
        } else {
            eprintln!("bacon configuration file already exists.");
        }
        println!("{}", package_config_path.to_string_lossy());
        return Ok(());
    }

    #[cfg(unix)]
    if let Some(action) = &args.send {
        socket::send_action(&context, action)?;
        return Ok(());
    }

    let settings = Settings::read(&args, &context)?;

    if args.list_jobs {
        print_jobs(&settings);
        return Ok(());
    }
    if args.completion_list_jobs {
        for job in settings.jobs.keys() {
            print!("{job}\0");
        }
        return Ok(());
    }

    let mut w = writer();
    if !headless {
        w.queue(EnterAlternateScreen)?;
        w.queue(cursor::Hide)?;
        #[cfg(windows)]
        w.queue(EnableMouseCapture)?;
        w.flush()?;
    }
    let result = tui::app::run(&mut w, settings, &args, context, headless);
    if !headless {
        #[cfg(windows)]
        w.queue(DisableMouseCapture)?;
        w.queue(cursor::Show)?;
        w.queue(LeaveAlternateScreen)?;
    }
    w.flush()?;
    result
}
