use {
    schemars::JsonSchema,
    serde::Deserialize,
};

/// Strategy to apply when changes are detected while a job is running.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OnChangeStrategy {
    /// Stop the running job immediately before starting a new one.
    KillThenRestart,
    /// Let the running job finish before starting again.
    WaitThenRestart,
}
