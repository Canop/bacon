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
    pub lines: Vec<CommandOutputLine>,
}

impl HasWrappableLines for CommandOutput {
    type WL = CommandOutputLine;
    fn wrappable_lines(&self) -> &[Self::WL] {
        &self.lines
    }
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

impl WrappableLine for CommandOutputLine {
    fn content(&self) -> &TLine {
        &self.content
    }
    fn prefix_cols(&self) -> usize {
        0
    }
}

impl CommandOutput {
    pub fn reverse(&mut self) {
        self.lines.reverse()
    }
    pub fn push(
        &mut self,
        line: CommandOutputLine,
    ) {
        self.lines.push(line);
    }
    pub fn len(&self) -> usize {
        self.lines.len()
    }
}

impl CommandOutputLine {
    pub fn matches(
        &self,
        pattern: Option<&str>,
    ) -> bool {
        if let Some(pattern) = pattern {
            if !self.content.has(pattern) {
                return false;
            }
        }
        true
    }
}
