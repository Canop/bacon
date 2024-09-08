use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OnChangeStrategy {
    KillThenRestart,
    WaitThenRestart,
}
