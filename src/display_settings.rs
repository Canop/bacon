use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct DisplaySettings {
    pub summary: bool,
    pub wrap: bool,
    pub reverse: bool,
}

impl DisplaySettings {
    pub fn apply_prefs(&mut self, prefs: &Prefs) {
        if let Some(b) = prefs.summary {
            self.summary = b;
        }
        if let Some(b) = prefs.wrap {
            self.wrap = b;
        }
    }
    pub fn apply_args(&mut self, args: &Args) {
        if args.no_summary {
            self.summary = false;
        }
        if args.summary {
            self.summary = true;
        }
        if args.no_wrap {
            self.wrap = false;
        }
        if args.wrap {
            self.wrap = true;
        }
    }
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            summary: false,
            wrap: false,
            reverse: true,
        }
    }
}
