use {
    argh::FromArgs,
    std::path::PathBuf,
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

    /// start in summary mode
    #[argh(switch, short = 's')]
    pub summary: bool,

    /// start with lines wrapped
    #[argh(switch, short = 'w')]
    pub wrap: bool,

    /// run `cargo clippy` instead of `cargo check`
    #[argh(switch, short = 'c')]
    pub clippy: bool,

    #[argh(positional)]
    /// path to the folder to watch or to the Rust repository
    pub root: Option<PathBuf>,
}

