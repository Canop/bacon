use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct DisplaySettings {
    pub summary: bool,
}

impl DisplaySettings {
    pub fn from(args: &Args) -> Self {
        Self {
            summary: args.summary,
        }
    }
}
