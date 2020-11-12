

pub static DEFAULT_PACKAGE_CONFIG: &str = r#"
# This is a configuration file for the bacon tool
# More info at https://github.com/Canop/bacon

default_job = "check"

[jobs]

[jobs.check]
command = ["cargo", "check", "--color", "always"]

[jobs.clippy]
command = ["cargo", "clippy", "--color", "always"]

"#;
