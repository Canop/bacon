use crate::Period;

/// Settings for one execution of a job's command
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Task {
    pub backtrace: Option<&'static str>, // ("1" or "full")
    pub grace_period: Period,
}
