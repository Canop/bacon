use crate::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandStream {
    StdOut,
    StdErr,
}

/// a line coming either from stdout or from stderr
#[derive(Debug, Clone)]
pub struct CommandOutputLine {
    pub content: TLine,
    pub origin: CommandStream,
}

