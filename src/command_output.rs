use {
    crate::*,
    std::process::ExitStatus,
};

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

/// a piece of information about the execution of a command
pub enum CommandExecInfo {
    /// Command ended
    End { status: Option<ExitStatus> },

    /// Bacon killed the command
    Interruption,

    /// Execution failed
    Error(String),

    /// Here's a line of output (coming from stderr or stdout)
    Line(CommandOutputLine),
}
