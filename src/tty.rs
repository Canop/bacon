use {
    crate::*,
    anyhow::*,
    std::{
        fmt::Write as _,
        io::Write,
    },
    termimad::StrFit,
};

pub const CSI_RESET: &str = "\u{1b}[0m\u{1b}[0m";
pub const CSI_BOLD: &str = "\u{1b}[1m";
pub const CSI_ITALIC: &str = "\u{1b}[3m";

pub const CSI_GREEN: &str = "\u{1b}[32m";

pub const CSI_RED: &str = "\u{1b}[31m";
pub const CSI_BOLD_RED: &str = "\u{1b}[1m\u{1b}[38;5;9m";
pub const CSI_BOLD_ORANGE: &str = "\u{1b}[1m\u{1b}[38;5;208m";

/// Used for "Blocking"
pub const CSI_BLUE: &str = "\u{1b}[1m\u{1b}[36m";

#[cfg(windows)]
pub const CSI_BOLD_YELLOW: &str = "\u{1b}[1m\u{1b}[38;5;11m";
#[cfg(not(windows))]
pub const CSI_BOLD_YELLOW: &str = "\u{1b}[1m\u{1b}[33m";

#[cfg(windows)]
pub const CSI_BOLD_BLUE: &str = "\u{1b}[1m\u{1b}[38;5;14m";
#[cfg(not(windows))]
pub const CSI_BOLD_BLUE: &str = "\u{1b}[1m\u{1b}[38;5;12m";

#[cfg(windows)]
pub const CSI_BOLD_4BIT_YELLOW: &str = "\u{1b}[1m\u{1b}[33m";

#[cfg(windows)]
pub const CSI_BOLD_WHITE: &str = "\u{1b}[1m\u{1b}[38;5;15m";

static TAB_REPLACEMENT: &str = "    ";

/// a simple representation of a colored and styled string.
///
/// Note that this works because of a few properties of
/// cargo's output:
/// - styles and colors are always reset on changes
/// - they're always in the same order (bold then fg color)
/// A more generic parsing would have to:
/// - parse the csi params (it's simple enough to map but takes code)
/// - use a simple state machine to keep style (bold, italic, etc.),
///    foreground color, and background color across tstrings
#[derive(Debug, Default, Clone)]
pub struct TString {
    pub csi: String,
    pub raw: String,
}
impl TString {
    /// colors are 8bits ansi values
    pub fn badge(
        con: &str,
        fg: u8,
        bg: u8,
    ) -> Self {
        Self {
            csi: format!("\u{1b}[1m\u{1b}[38;5;{}m\u{1b}[48;5;{}m", fg, bg),
            raw: format!(" {} ", con),
        }
    }
    pub fn num_badge(
        num: usize,
        cat: &str,
        fg: u8,
        bg: u8,
    ) -> Self {
        let raw = if num == 1 {
            format!(" 1 {} ", cat)
        } else {
            format!(" {} {}s ", num, cat)
        };
        Self::badge(&raw, fg, bg)
    }
    pub fn push_csi(
        &mut self,
        params: &[i64],
        action: char,
    ) {
        self.csi.push('\u{1b}');
        self.csi.push('[');
        for (idx, p) in params.iter().enumerate() {
            let _ = write!(self.csi, "{}", p);
            if idx < params.len() - 1 {
                self.csi.push(';');
            }
        }
        self.csi.push(action);
    }
    pub fn draw(
        &self,
        w: &mut W,
    ) -> Result<()> {
        if self.csi.is_empty() {
            write!(w, "{}", &self.raw)?;
        } else {
            write!(w, "{}{}{}", &self.csi, &self.raw, CSI_RESET,)?;
        }
        Ok(())
    }
    /// draw the string but without taking more than cols_max cols.
    /// Return the number of cols written
    pub fn draw_in(
        &self,
        w: &mut W,
        cols_max: usize,
    ) -> Result<usize> {
        let fit = StrFit::make_cow(&self.raw, cols_max);
        if self.csi.is_empty() {
            write!(w, "{}", &fit.0)?;
        } else {
            write!(w, "{}{}{}", &self.csi, &fit.0, CSI_RESET,)?;
        }
        Ok(fit.1)
    }
    pub fn starts_with(
        &self,
        csi: &str,
        raw: &str,
    ) -> bool {
        self.csi == csi && self.raw.starts_with(raw)
    }
    pub fn split_off(
        &mut self,
        at: usize,
    ) -> Self {
        Self {
            csi: self.csi.clone(),
            raw: self.raw.split_off(at),
        }
    }
}

/// a simple representation of a line made of homogeneous parts.
///
/// Note that this only manages CSI and SGR components
/// and isn't a suitable representation for an arbitrary
/// terminal input or output.
/// I recommend you to NOT try to reuse this hack in another
/// project unless you perfectly understand it.
#[derive(Debug, Default, Clone)]
pub struct TLine {
    pub strings: Vec<TString>,
}

impl TLine {
    pub fn from_tty(tty: &str) -> Self {
        let tty_str: String;
        let tty = if tty.contains('\t') {
            tty_str = tty.replace('\t', TAB_REPLACEMENT);
            &tty_str
        } else {
            tty
        };
        let mut parser = vte::Parser::new();
        let mut builder = TLineBuilder::default();
        for byte in tty.bytes() {
            parser.advance(&mut builder, byte);
        }
        builder.into_tline()
    }
    pub fn from_raw(raw: String) -> Self {
        Self {
            strings: vec![TString {
                csi: " ".to_string(),
                raw,
            }],
        }
    }
    pub fn to_raw(&self) -> String {
        let mut s = String::new();
        for ts in &self.strings {
            s.push_str(&ts.raw);
        }
        s
    }
    pub fn bold(raw: String) -> Self {
        Self {
            strings: vec![TString {
                csi: CSI_BOLD.to_string(),
                raw,
            }],
        }
    }
    pub fn italic(raw: String) -> Self {
        Self {
            strings: vec![TString {
                csi: CSI_ITALIC.to_string(),
                raw,
            }],
        }
    }
    pub fn failed(key: &str) -> Self {
        Self {
            strings: vec![
                TString {
                    csi: CSI_BOLD_ORANGE.to_string(),
                    raw: "failed".to_string(),
                },
                TString {
                    csi: CSI_BOLD.to_string(),
                    raw: format!(": {}", key),
                },
            ],
        }
    }
    pub fn add_badge(
        &mut self,
        badge: TString,
    ) {
        self.strings.push(badge);
        self.strings.push(TString {
            csi: "".to_string(),
            raw: " ".to_string(),
        });
    }
    pub fn draw(
        &self,
        w: &mut W,
    ) -> Result<()> {
        for ts in &self.strings {
            ts.draw(w)?;
        }
        Ok(())
    }
    /// draw the line but without taking more than cols_max cols.
    /// Return the number of cols written
    pub fn draw_in(
        &self,
        w: &mut W,
        cols_max: usize,
    ) -> Result<usize> {
        let mut cols = 0;
        for ts in &self.strings {
            if cols >= cols_max {
                break;
            }
            cols += ts.draw_in(w, cols_max - cols)?;
        }
        Ok(cols)
    }
    pub fn is_blank(&self) -> bool {
        return self.strings.iter().all(|s| s.raw.trim().is_empty());
    }
    // if this line has no style, return its content
    pub fn if_unstyled(&self) -> Option<&str> {
        if self.strings.len() == 1 {
            self.strings
                .get(0)
                .filter(|s| s.csi.is_empty())
                .map(|s| s.raw.as_str())
        } else {
            None
        }
    }
    pub fn has(
        &self,
        part: &str,
    ) -> bool {
        self.strings.iter().any(|s| s.raw.contains(part))
    }
}

#[derive(Debug, Default)]
pub struct TLineBuilder {
    cur: Option<TString>,
    strings: Vec<TString>,
}
impl TLineBuilder {
    pub fn into_tline(mut self) -> TLine {
        if let Some(cur) = self.cur {
            self.strings.push(cur);
        }
        TLine {
            strings: self.strings,
        }
    }
}
impl vte::Perform for TLineBuilder {
    fn print(
        &mut self,
        c: char,
    ) {
        self.cur.get_or_insert_with(TString::default).raw.push(c);
    }
    fn csi_dispatch(
        &mut self,
        params: &[i64],
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        if *params == [0] {
            if let Some(cur) = self.cur.take() {
                self.strings.push(cur);
            }
            return;
        }
        if let Some(cur) = self.cur.as_mut() {
            if cur.raw.is_empty() {
                cur.push_csi(params, action);
                return;
            }
        }
        if let Some(cur) = self.cur.take() {
            self.strings.push(cur);
        }
        let mut cur = TString::default();
        cur.push_csi(params, action);
        self.cur = Some(cur);
    }
    fn execute(
        &mut self,
        _byte: u8,
    ) {
    }
    fn hook(
        &mut self,
        _params: &[i64],
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
    }
    fn put(
        &mut self,
        _byte: u8,
    ) {
    }
    fn unhook(&mut self) {}
    fn osc_dispatch(
        &mut self,
        _params: &[&[u8]],
        _bell_terminated: bool,
    ) {
    }
    fn esc_dispatch(
        &mut self,
        _intermediates: &[u8],
        _ignore: bool,
        _byte: u8,
    ) {
    }
}
