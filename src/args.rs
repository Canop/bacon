use {
    anyhow::{bail, Result},
    clap::Parser,
};

#[derive(Debug, Parser)]
/// bacon watches your source and run code checks in background.
///
/// Documentation at <https://dystroy.org/bacon>
#[clap(version, about)]
pub struct Args {

    /// print the path to the prefs file, create it if it doesn't exist
    #[clap(long = "prefs")]
    pub prefs: bool,

    /// start in summary mode
    #[clap(short = 's', long = "summary")]
    pub summary: bool,

    /// start in full mode (not summary)
    #[clap(short = 'S', long = "no-summary")]
    pub no_summary: bool,

    /// start with lines wrapped
    #[clap(short = 'w', long = "wrap")]
    pub wrap: bool,

    /// start with lines not wrapped
    #[clap(short = 'W', long = "no-wrap")]
    pub no_wrap: bool,

    /// start with gui vertical order reversed
    #[clap(long = "reverse")]
    pub reverse: bool,

    /// start with standard gui order (focus on top)
    #[clap(long = "no-reverse")]
    pub no_reverse: bool,

    /// list available jobs
    #[clap(short = 'l', long = "list-jobs")]
    pub list_jobs: bool,

    /// create a bacon.toml file, ready to be customized
    #[clap(long = "init")]
    pub init: bool,

    /// job to launch ("check", "clippy", customized ones, ...)
    #[clap(short = 'j', long = "job")]
    pub job: Option<String>,

    /// ignore features of both the package and the bacon job
    #[clap(long = "no-default-features")]
    pub no_default_features: bool,

    /// check all members in the workspace.
    ///
    /// Equivalent to `cargo check --workspace`
    #[clap(long = "workspace")]
    pub workspace: bool,

    /// activate all available features
    #[clap(long = "all-features")]
    pub all_features: bool,

    /// comma separated list of features to ask cargo to compile with
    /// (if the job defines some, they're merged)
    #[clap(long = "features")]
    pub features: Option<String>,

    /// path to watch (must be a rust directory or inside)
    #[clap(short = 'p', long = "path")]
    pub path: Option<String>,

    #[clap()]
    /// either a job, or a path, or both
    pub args: Vec<String>,

    #[clap(last = true)]
    /// arguments given to the job
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
                    self.job = b;
                } else {
                    self.job = Some(a);
                    self.path = b;
                }
            }
            (Some(_), Some(_), _, _) => {
                bail!("Too many arguments");
            }
            (Some(a), None, true, false) => {
                self.job = Some(a);
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
}
