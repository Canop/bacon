use super::*;

/// A builder consuming a string assumed to contain TTY sequences and building a TLine.
#[derive(Debug, Default)]
pub struct TLineBuilder {
    cur: Option<TString>,
    strings: Vec<TString>,
}
impl TLineBuilder {
    pub fn read(
        &mut self,
        s: &str,
    ) {
        let mut parser = vte::Parser::new();
        for byte in s.bytes() {
            parser.advance(self, byte);
        }
    }
    pub fn build(mut self) -> TLine {
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
