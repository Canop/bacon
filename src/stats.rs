use crate::*;

/// number of lines per type in a report
#[derive(Debug, Default)]
pub struct Stats {
    pub warnings: usize,
    pub errors: usize,
    pub test_fails: usize,
    pub passed_tests: usize,
    pub location_lines: usize,
    pub normal_lines: usize,
}
impl From<&Vec<Line>> for Stats {
    fn from(lines: &Vec<Line>) -> Self {
        lines.iter().fold(Stats::default(), |mut stats, line| {
            match line.line_type {
                LineType::Title(Kind::Error) => stats.errors += 1,
                LineType::Title(Kind::Warning) => stats.warnings += 1,
                LineType::Title(Kind::TestFail) => stats.test_fails += 1,
                LineType::Location => stats.location_lines += 1,
                _ => stats.normal_lines += 1,
            }
            stats
        })
    }
}
impl Stats {
    pub fn lines(&self, summary: bool) -> usize {
        let mut sum = self.warnings + self.errors + self.test_fails + self.location_lines;
        if !summary {
            sum += self.normal_lines;
        }
        sum
    }
    pub fn items(&self) -> usize {
        self.warnings + self.errors + self.test_fails
    }
}
