use {
    crate::*,
    anyhow::{
        Result,
        bail,
    },
    clap::{
        CommandFactory,
        Parser,
    },
    clap_complete::ArgValueCandidates,
    termimad::ansi,
};

static INTRO: &str = "

**bacon** watches your project and runs jobs in background.

Use shortcuts to:
* switch job: *t* for `test`, *c* for `clippy`, *d* to open rust doc, etc.
* toggle display: *s* for summary, *w* for wrapped lines, etc.
* search: */*
* see all shortcuts: *?*
Complete documentation at https://dystroy.org/bacon
";

/// Launch arguments
#[derive(Debug, Parser)]
#[command(
    author,
    about,
    version,
    disable_version_flag = true,
    disable_help_flag = true
)]
pub struct Args {
    /// Print help information
    #[arg(long)]
    pub help: bool,

    /// Print the version
    #[arg(long)]
    pub version: bool,

    /// Print the path to the prefs file, create it if it doesn't exist
    #[clap(long)]
    pub prefs: bool,

    /// Run without user interface: just run the default job on change
    #[clap(long)]
    pub headless: bool,

    /// Start in summary mode
    #[clap(short = 's', long)]
    pub summary: bool,

    /// Start in full mode (not summary)
    #[clap(short = 'S', long)]
    pub no_summary: bool,

    /// Start with lines wrapped
    #[clap(short = 'w', long)]
    pub wrap: bool,

    /// Start with lines not wrapped
    #[clap(short = 'W', long)]
    pub no_wrap: bool,

    /// Start with gui vertical order reversed
    #[clap(long)]
    pub reverse: bool,

    /// Start with standard gui order
    #[clap(long)]
    pub no_reverse: bool,

    /// Display a help line
    #[clap(long)]
    pub help_line: bool,

    /// Do not display a help line
    #[clap(long)]
    pub no_help_line: bool,

    /// List available jobs
    #[clap(short = 'l', long)]
    pub list_jobs: bool,
    #[clap(long, hide = true)]
    pub completion_list_jobs: bool,

    /// Don't access the network
    #[clap(long)]
    pub offline: bool,

    /// Create a bacon.toml file, ready to be customized
    #[clap(long)]
    pub init: bool,

    /// Listen on a unix socket for commands
    #[cfg(unix)]
    #[clap(long)]
    pub listen: bool,

    /// Don't listen on a unix socket for commands
    #[cfg(unix)]
    #[clap(long)]
    pub no_listen: bool,

    /// Send a command on bacon's unix socket
    #[cfg(unix)]
    #[clap(long)]
    pub send: Option<String>,

    /// Job to launch: `check`, `clippy`, custom ones...
    #[clap(short = 'j', long, value_name = "job", add = ArgValueCandidates::new(crate::cli::completions::list_jobs))]
    pub job: Option<ConcreteJobRef>,

    /// Ignore features of both the package and the bacon job
    #[clap(long)]
    pub no_default_features: bool,

    /// Comma separated list of features
    /// (if the job defines some, they're merged)
    #[clap(long, value_name = "features")]
    pub features: Option<String>,

    /// Activate all available features
    #[clap(long)]
    pub all_features: bool,

    /// Export locations in `.bacon-locations` file
    #[clap(short = 'e', long)]
    pub export_locations: bool,

    /// Don't export locations
    #[clap(short = 'E', long)]
    pub no_export_locations: bool,

    /// Path to watch (overriding what's normally computed from the project's
    /// type, bacon.toml file, etc.)
    #[clap(long, value_name = "watch", value_hint = clap::ValueHint::FilePath)]
    pub watch: Option<String>,

    /// Project to run jobs on, and use as working directory
    #[clap(long, value_name = "project", value_hint = clap::ValueHint::DirPath)]
    pub project: Option<String>,

    /// Configuration passed as a TOML string
    #[clap(long)]
    pub config_toml: Option<String>,

    /// Generate the JSON Schema for bacon configuration files
    #[clap(long)]
    pub generate_config_schema: bool,

    #[clap(add = ArgValueCandidates::new(crate::cli::completions::list_jobs))]
    /// What to do: either a job, or a path, or both
    pub args: Vec<String>,

    #[clap(last = true)]
    /// Arguments given to the job
    pub additional_job_args: Vec<String>,
}

impl Args {
    /// positional arguments in bacon command are a convenience
    /// allowing to skip writing `-j`, `-p`, or both.
    /// To be used, they must be copied to the `job` or
    /// `path` values.
    pub fn fix(&mut self) -> Result<()> {
        let mut args = self.args.drain(..);
        match (
            args.next(),
            args.next(),
            self.job.is_none(),
            self.project.is_none(),
        ) {
            (Some(a), b, true, true) => {
                if a.contains('.') || a.contains('/') {
                    // a is a path, it can't be job
                    self.project = Some(a);
                    self.job = b.map(|b| b.as_str().into());
                } else {
                    self.job = Some(a.as_str().into());
                    self.project = b;
                }
            }
            (Some(_), Some(_), _, _) => {
                bail!("Too many arguments");
            }
            (Some(a), None, true, false) => {
                self.job = Some(a.as_str().into());
            }
            (Some(a), None, false, true) => {
                self.project = Some(a);
            }
            (Some(a), None, false, false) => {
                bail!("Unexpected argument {a:?}");
            }
            _ => {}
        }
        Ok(())
    }
    pub fn print_help(&self) {
        let mut printer = clap_help::Printer::new(Args::command())
            .with("introduction", INTRO)
            .with("options", clap_help::TEMPLATE_OPTIONS_MERGED_VALUE)
            .without("author");
        let skin = printer.skin_mut();
        skin.headers[0].compound_style.set_fg(ansi(204));
        skin.bold.set_fg(ansi(204));
        skin.italic = termimad::CompoundStyle::with_fg(ansi(204));
        printer.template_keys_mut().push("examples");
        printer.set_template("examples", EXAMPLES_TEMPLATE);
        for (i, example) in EXAMPLES.iter().enumerate() {
            printer
                .expander_mut()
                .sub("examples")
                .set("example-number", i + 1)
                .set("example-title", example.title)
                .set("example-cmd", example.cmd);
        }
        printer.print_help();
    }
}
