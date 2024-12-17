use {
    crate::*,
    serde::{
        Deserialize,
        Serialize,
    },
    std::process::ExitStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq)]
pub enum CommandStream {
    StdOut,
    StdErr,
}

/// a line coming either from stdout or from stderr
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandOutputLine {
    pub content: TLine,
    pub origin: CommandStream,
}

/// some output lines
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandOutput {
    pub lines: Vec<Line>,
}

/// a piece of information about the execution of a command
#[derive(Debug)]
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

impl CommandOutput {
    pub fn reverse(&mut self) {
        self.lines.reverse()
    }
    pub fn push<L: Into<Line>>(
        &mut self,
        line: L,
    ) {
        self.lines.push(line.into());
    }
    pub fn len(&self) -> usize {
        self.lines.len()
    }
}
