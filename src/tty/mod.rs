mod tline;
mod tline_builder;
mod trange;
mod tstring;

pub const CSI_RESET: &str = "\u{1b}[0m";
pub const CSI_BOLD: &str = "\u{1b}[1m";
pub const CSI_ITALIC: &str = "\u{1b}[3m";

pub const CSI_GREEN: &str = "\u{1b}[32m";

pub const CSI_RED: &str = "\u{1b}[31m";
pub const CSI_BOLD_RED: &str = "\u{1b}[1m\u{1b}[38;5;9m";
pub const CSI_BOLD_ORANGE: &str = "\u{1b}[1m\u{1b}[38;5;208m";
pub const CSI_BOLD_GREEN: &str = "\u{1b}[1m\u{1b}[38;5;34m";

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

use {
    crate::W,
    anyhow::Result,
    std::io::Write,
};

pub use {
    tline::*,
    tline_builder::*,
    trange::*,
    tstring::*,
};

pub fn draw(
    w: &mut W,
    csi: &str,
    raw: &str,
) -> Result<()> {
    if csi.is_empty() {
        write!(w, "{raw}")?;
    } else {
        write!(w, "{csi}{raw}{CSI_RESET}")?;
    }
    Ok(())
}
pub fn csi(
    fg: u8,
    bg: u8,
) -> String {
    format!("\u{1b}[1m\u{1b}[38;5;{fg}m\u{1b}[48;5;{bg}m")
}
