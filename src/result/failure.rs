use {
    crate::*,
    serde::{
        Deserialize,
        Serialize,
    },
};

/// data of a failed command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Failure {
    pub error_code: i32,
    pub output: CommandOutput,
    pub suggest_backtrace: bool,
}
