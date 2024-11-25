use {
    super::*,
    crate::*,
    anyhow::*,
    serde::{
        Deserialize,
        Serialize,
    },
};

/// a simple representation of a line made of homogeneous parts.
///
/// Note that this only manages CSI and SGR components
/// and isn't a suitable representation for an arbitrary
/// terminal input or output.
/// I recommend you to NOT try to reuse this hack in another
/// project unless you perfectly understand it.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
        let mut builder = TLineBuilder::default();
        builder.read(tty);
        builder.build()
    }
    //pub fn ends_with_newline(
    //    &self,
    //) -> bool {
    //    self.strings.last().map_or(false, |s| s.raw.ends_with('\n'))
    //}
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
        let mut strings = Vec::with_capacity(4);
        strings.push(TString::new(CSI_BOLD_ORANGE, "failed"));
        strings.push(TString::new("", ": "));
        if let Some((module, function)) = key.rsplit_once("::") {
            strings.push(TString {
                csi: "".to_string(),
                raw: format!("{module}::"),
            });
            strings.push(TString::new(CSI_BOLD_ORANGE, function));
        } else {
            strings.push(TString::new(CSI_BOLD_ORANGE, key));
        }
        TLine { strings }
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
