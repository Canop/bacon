use crate::*;

/// data of a failed command
pub struct Failure {
    pub error_code: i32,
    pub output: CommandOutput,
}
