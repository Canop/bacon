use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub summary: bool,
    pub wrap: bool,
    pub reverse: bool,
    pub vim_keys: bool,
}

impl Settings {
    pub fn apply_prefs(&mut self, prefs: &Prefs) {
        if let Some(b) = prefs.summary {
            self.summary = b;
        }
        if let Some(b) = prefs.wrap {
            self.wrap = b;
        }
        if let Some(b) = prefs.reverse {
            self.reverse = b;
        }
        if let Some(b) = prefs.vim_keys {
            self.vim_keys = b;
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
        if args.no_reverse {
            self.reverse = false;
        }
        if args.reverse {
            self.reverse = true;
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            summary: false,
            wrap: false,
            reverse: false,
            vim_keys: false,
        }
    }
}
