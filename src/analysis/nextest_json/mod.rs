use crate::{CommandOutput, CommandOutputLine, CommandStream, Report, TLine};

use super::{Analyzer, LineType, Stats, standard::StandardAnalyzer};

#[derive(Debug, Default)]
pub struct NextestJSONAnalyzer {
    lines: Vec<CommandOutputLine>,
    standard_analyzer: StandardAnalyzer,
}

impl Analyzer for NextestJSONAnalyzer {
    fn start(
        &mut self,
        _mission: &crate::Mission,
    ) {
        self.lines.clear();
    }

    fn receive_line(
        &mut self,
        line: CommandOutputLine,
        command_output: &mut crate::CommandOutput,
    ) {
        match line.origin {
            // In JSON mode, stderr output is human readable
            CommandStream::StdErr => self.standard_analyzer.receive_line(line, command_output),

            // The JSON payloads come on stdout
            CommandStream::StdOut => self.lines.push(line),
        }
    }

    fn build_report(&mut self) -> anyhow::Result<crate::Report> {
        let mut report = Report {
            lines: Vec::with_capacity(self.lines.len()),
            stats: Stats::default(),
            suggest_backtrace: false,
            output: CommandOutput::default(),
            failure_keys: Vec::default(),
        };

        let mut item_idx = 0;
        for line in &self.lines {
            let line: OutputLine = serde_json::from_str(&line.content.to_raw())?;
            match line {
                OutputLine::Suite { event } => match event {
                    SuiteEvent::Started { test_count: _ } => (),
                    SuiteEvent::Ok { .. } => (),
                    SuiteEvent::Failed {
                        passed,
                        failed,
                        ignored: _,
                        measured: _,
                        filtered_out: _,
                        exec_time: _,
                    } => {
                        report.stats.test_fails = failed;
                        report.stats.passed_tests = passed;
                    }
                },
                OutputLine::Test { event } => match event {
                    TestEvent::Started => (),
                    TestEvent::Ok { name: _ } => (),
                    TestEvent::Ignored { name: _ } => (),
                    TestEvent::Failed { name, stdout } => {
                        let name = cleanup_name(&name);
                        report.lines.push(crate::Line {
                            item_idx,
                            line_type: LineType::Title(super::Kind::Error),
                            content: TLine::failed(&name),
                        });
                        for outline in stdout.lines() {
                            report.lines.push(crate::Line {
                                item_idx,
                                line_type: LineType::Normal,
                                content: TLine::from_raw(outline.to_owned()),
                            });
                        }
                        report.failure_keys.push(name);
                        item_idx += 1;
                    }
                },
            }
        }
        Ok(report)
    }
}

fn cleanup_name(name: &str) -> String {
    if let Some(idx) = name.chars().position(|ch| ch == '$') {
        name.chars().skip(idx + 1).collect()
    } else {
        name.to_owned()
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "event")]
enum TestEvent {
    Started,
    Ok { name: String },
    Failed { name: String, stdout: String },
    Ignored { name: String },
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "event")]
enum SuiteEvent {
    Started {
        test_count: usize,
    },
    Ok {
        passed: usize,
        failed: usize,
        ignored: usize,
        measured: usize,
        filtered_out: usize,
        exec_time: f32,
    },
    Failed {
        passed: usize,
        failed: usize,
        ignored: usize,
        measured: usize,
        filtered_out: usize,
        exec_time: f32,
    },
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
enum OutputLine {
    Suite {
        #[serde(flatten)]
        event: SuiteEvent,
    },
    Test {
        #[serde(flatten)]
        event: TestEvent,
    },
}

#[cfg(test)]
mod test {
    use super::{OutputLine, SuiteEvent, TestEvent};

    #[test]
    fn parse() {
        let suite_started = r#"{"type":"suite","event":"started","test_count":7}"#;
        assert_eq!(
            serde_json::from_str::<OutputLine>(suite_started).unwrap(),
            OutputLine::Suite {
                event: SuiteEvent::Started { test_count: 7 }
            }
        );

        let suite_failed = r#"{"type":"suite","event":"failed","passed":6,"failed":1,"ignored":0,"measured":0,"filtered_out":0,"exec_time":0.015213355}"#;
        assert_eq!(
            serde_json::from_str::<OutputLine>(suite_failed).unwrap(),
            OutputLine::Suite {
                event: SuiteEvent::Failed {
                    passed: 6,
                    failed: 1,
                    ignored: 0,
                    measured: 0,
                    filtered_out: 0,
                    exec_time: 0.015213355
                }
            }
        );

        let suite_ok = r#"{"type":"suite","event":"ok","passed":13,"failed":0,"ignored":0,"measured":0,"filtered_out":0,"exec_time":0.140588}"#;

        assert_eq!(
            serde_json::from_str::<OutputLine>(suite_ok).unwrap(),
            OutputLine::Suite {
                event: SuiteEvent::Ok {
                    passed: 13,
                    failed: 0,
                    ignored: 0,
                    measured: 0,
                    filtered_out: 0,
                    exec_time: 0.140588
                }
            }
        );

        let test_started =
            r#"{"type":"test","event":"started","name":"llm::llm$parser::test::number"}"#;
        assert_eq!(
            serde_json::from_str::<OutputLine>(test_started).unwrap(),
            OutputLine::Test {
                event: TestEvent::Started {}
            }
        );

        let test_ok = r#"{"type":"test","event":"ok","name":"llm::llm$parser::test::identifier","exec_time":0.002138244}"#;

        assert_eq!(
            serde_json::from_str::<OutputLine>(test_ok).unwrap(),
            OutputLine::Test {
                event: TestEvent::Ok {
                    name: "llm::llm$parser::test::identifier".to_owned(),
                }
            }
        );

        let test_fail = r#" {"type":"test","event":"failed","name":"llm::llm$parser::test::var","exec_time":0.002140747,"stdout":"thread 'parser::test::var' panicked at src"}"#;
        assert_eq!(
            serde_json::from_str::<OutputLine>(test_fail).unwrap(),
            OutputLine::Test {
                event: TestEvent::Failed {
                    name: "llm::llm$parser::test::var".to_owned(),
                    stdout: "thread 'parser::test::var' panicked at src".to_string(),
                }
            }
        );

        let test_ignored =
            r#"{"type":"test","event":"ignored","name":"llvm::llvm$parser::test::var"}"#;
        assert_eq!(
            serde_json::from_str::<OutputLine>(test_ignored).unwrap(),
            OutputLine::Test {
                event: TestEvent::Ignored {
                    name: "llm::llm$parser::test::var".to_owned(),
                }
            }
        );
    }

    #[test]
    fn cleanup_name() {
        let name = "bacon::bacon$analysis::nextest_json::test_fail";
        assert_eq!(
            super::cleanup_name(name),
            "analysis::nextest_json::test_fail".to_owned()
        );
    }
}
