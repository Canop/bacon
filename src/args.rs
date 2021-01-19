use {
    anyhow::*,
    argh::FromArgs,
};

#[derive(Debug, FromArgs)]
/// bacon watches your source and run code checks in background.
///
///
/// Source at https://github.com/Canop/bacon
pub struct Args {
    /// print the version
    #[argh(switch, short = 'v')]
    pub version: bool,

    /// print the path to the prefs file, create it if it doesn't exist
    #[argh(switch)]
    pub prefs: bool,

    /// start in summary mode
    #[argh(switch, short = 's')]
    pub summary: bool,

    /// start in full mode (not summary)
    #[argh(switch, short = 'S')]
    pub no_summary: bool,

    /// start with lines wrapped
    #[argh(switch, short = 'w')]
    pub wrap: bool,

    /// start with lines not wrapped
    #[argh(switch, short = 'W')]
    pub no_wrap: bool,

    /// start with gui vertical order reversed
    #[argh(switch)]
    pub reverse: bool,

    /// start with standard gui order (focus on top)
    #[argh(switch)]
    pub no_reverse: bool,

    /// create a bacon.toml file, ready to be customized
    #[argh(switch)]
    pub init: bool,

    /// job to launch ("check", "clippy", customized ones, ...)
    #[argh(option, short = 'j')]
    pub job: Option<String>,

    /// ignore features of both the package and the bacon job
    #[argh(switch)]
    pub no_default_features: bool,

    /// comma separated list of features to ask cargo to compile with
    /// (if the job defines some, they're merged)
    #[argh(option)]
    pub features: Option<String>,

    /// path to watch (must be a rust directory or inside)
    #[argh(option, short = 'p')]
    pub path: Option<String>,

    #[argh(positional)]
    /// either a job, or a path, or both
    pub args: Vec<String>,
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
                return Err(anyhow!("Too many arguments"));
            }
            (Some(a), None, true, false) => {
                self.job = Some(a);
            }
            (Some(a), None, false, true) => {
                self.path = Some(a);
            }
            (Some(a), None, false, false) => {
                return Err(anyhow!("Unexpected argument {:?}", a));
            }
            _ => {}
        }
        Ok(())
    }
}
