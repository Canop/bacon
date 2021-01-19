use crate::*;

#[derive(Debug, Clone)]
pub struct Settings {
    pub summary: bool,
    pub wrap: bool,
    pub reverse: bool,
    pub vim_keys: bool,
    pub no_default_features: bool,
    pub features: Option<String>, // comma separated list
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
        if args.no_default_features {
            self.no_default_features = true;
        }
        if args.features.is_some() {
            self.features = args.features.clone();
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
            no_default_features: false,
            features: None,
        }
    }
}
