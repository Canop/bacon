pub trait Md {
    fn md(&self) -> String;
}

impl Md for &str {
    fn md(&self) -> String {
        self.to_string()
    }
}
