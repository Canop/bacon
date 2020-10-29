


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub kind: Kind,
    pub lines: Vec<String>,
}
