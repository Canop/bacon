/// A dynamic reduction of a job execution
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Scope {
    pub tests: Vec<String>,
}

impl Scope {
    pub fn has_tests(&self) -> bool {
        !self.tests.is_empty()
    }
}
