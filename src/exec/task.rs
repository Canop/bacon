use std::time::Duration;

/// Settings for one execution of a job's command
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Task {
    pub backtrace: Option<&'static str>, // ("1" or "full")
    pub grace_period: Duration,
}
