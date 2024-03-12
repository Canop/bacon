use {
    crate::*,
    anyhow::{bail, Result},
    clap::{CommandFactory, Parser},
    termimad::ansi,
};

static INTRO: &str = "

**bacon** watches your rust project and runs jobs in background.

Use shortcuts to:
* switch job: *t* for `test`, *c* for `clippy`, *d* to open rust doc, etc.
* toggle display: *s* for summary, *w* for wrapped lines, etc.
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

    /// Don't access the network
    #[clap(long)]
    pub offline: bool,

    /// Create a bacon.toml file, ready to be customized
    #[clap(long)]
    pub init: bool,

    /// Job to launch: `check`, `clippy`, custom ones...
    #[clap(short = 'j', long, value_name = "job")]
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

    /// Path to watch (must be a rust directory or inside)
    #[clap(short = 'p', long, value_name = "path")]
    pub path: Option<String>,

    #[clap()]
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
            self.path.is_none(),
        ) {
            (Some(a), b, true, true) => {
                if a.contains('.') || a.contains('/') {
                    // a is a path, it can't be job
                    self.path = Some(a);
                    self.job = b.map(|b| b.as_str().into());
                } else {
                    self.job = Some(a.as_str().into());
                    self.path = b;
                }
            }
            (Some(_), Some(_), _, _) => {
                bail!("Too many arguments");
            }
            (Some(a), None, true, false) => {
                self.job = Some(a.as_str().into());
            }
            (Some(a), None, false, true) => {
                self.path = Some(a);
            }
            (Some(a), None, false, false) => {
                bail!("Unexpected argument {:?}", a);
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
        skin.italic = termimad::CompoundStyle::with_fg(ansi(204)); // 2, 81, 73, 38
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
