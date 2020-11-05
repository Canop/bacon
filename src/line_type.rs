
static STYLED_PREFIX: &str = "\u{1b}[0m";

/// "warning" in bold yellow, followed by a bold colon
static WARNING: &str = "\u{1b}[1m\u{1b}[33mwarning\u{1b}[0m\u{1b}[0m\u{1b}[1m: ";

/// "error" in bold red
static ERROR: &str = "\u{1b}[1m\u{1b}[38;5;9merror";

/// "error: aborting"
static ABORTING: &str = "\u{1b}[1m\u{1b}[38;5;9merror\u{1b}[0m\u{1b}[0m\u{1b}[1m: aborting";

/// a "-->" in bold blue
static ARROW: &str = "\u{1b}[0m\u{1b}[0m\u{1b}[1m\u{1b}[38;5;12m--> \u{1b}[0m\u{1b}[0m";

/// either Warning or Error
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineType {
    Title(Kind),
    Location,
    Normal,
    End,
}

/// check if the string starts with something like "15 warnings emitted"
fn is_n_warnings_emitted(s: &str) -> bool {
    let mut tokens = s.split_ascii_whitespace();
    if let Some(t) = tokens.next() {
        if t.parse::<usize>().is_err() {
            return false;
        }
        if let Some(t) = tokens.next() {
            if t != "warnings" && t != "warning" {
                return false;
            }
            if let Some(t) = tokens.next() {
                if t.starts_with("emitted") {
                    return true;
                }
            }
        }
    }
    false
}

impl From<&String> for LineType {
    fn from(content: &String) -> Self {
        // at some point we might have to import regex, or to look
        // for some cargo API, but for now it looks like it works and
        // avoids heavy dependencies
        if let Some(styled) = content.strip_prefix(STYLED_PREFIX) {
            let styled = styled.trim();
            if let Some(warning) = styled.strip_prefix(WARNING) {
                debug!("warning: {:?}", warning);
                if is_n_warnings_emitted(warning) {
                    LineType::End
                } else {
                    LineType::Title(Kind::Warning)
                }
            } else if styled.starts_with(ABORTING) {
                LineType::End
            } else if styled.starts_with(ERROR) {
                LineType::Title(Kind::Error)
            } else if styled.starts_with(ARROW) {
                LineType::Location
            } else {
                LineType::Normal
            }
        } else {
            LineType::Normal
        }
    }
}
