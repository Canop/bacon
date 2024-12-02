use {
    super::*,
    crate::*,
    anyhow::*,
    serde::{
        Deserialize,
        Serialize,
    },
    std::{
        fmt::Write as _,
        io::Write,
    },
    termimad::StrFit,
};

/// a simple representation of a colored and styled string.
///
/// Note that this works because of a few properties of
/// cargo's output:
/// - styles and colors are always reset on changes
/// - they're always in the same order (bold then fg color)
///
/// A more generic parsing would have to:
/// - parse the csi params (it's simple enough to map but takes code)
/// - use a simple state machine to keep style (bold, italic, etc.),
///    foreground color, and background color across tstrings
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TString {
    pub csi: String,
    pub raw: String,
}
impl TString {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        csi: S1,
        raw: S2,
    ) -> Self {
        Self {
            csi: csi.into(),
            raw: raw.into(),
        }
    }
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
        let raw = if num < 2 {
            format!(" {} {} ", num, cat)
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
            write!(w, "{}{}{}", &self.csi, &fit.0, CSI_RESET)?;
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
    pub fn is_blank(&self) -> bool {
        self.raw.chars().all(char::is_whitespace)
    }
    pub fn is_unstyled(&self) -> bool {
        self.csi.is_empty()
    }
}
