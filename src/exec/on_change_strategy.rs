use {
    schemars::JsonSchema,
    serde::Deserialize,
};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OnChangeStrategy {
    KillThenRestart,
    WaitThenRestart,
}
