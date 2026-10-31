use {
    crate::*,
    anyhow::{
        Result,
        anyhow,
    },
};

/// The stack of jobs that bacon ran, allowing to get back to the previous one,
/// or to scope the current one
#[derive(Default)]
pub struct JobStack {
    entries: Vec<ConcreteJobRef>,
}

impl JobStack {
    /// Apply the job ref instruction to determine the job to run, updating the stack.
    ///
    /// When `replace_current` is true, the current job is removed from the stack.
    ///
    /// When no job is returned, the application is supposed to quit.
    pub fn pick_job(
        &mut self,
        job_ref: &JobRef,
        replace_current: bool,
        settings: &Settings,
    ) -> Result<Option<(ConcreteJobRef, Job)>> {
        debug!("picking job {job_ref:?}");
        let current_len = self.entries.len();
        let concrete = match job_ref {
            JobRef::Default => settings.default_job.clone(),
            JobRef::Initial => settings
                .arg_job
                .as_ref()
                .unwrap_or(&settings.default_job)
                .clone(),
            JobRef::Previous | JobRef::PreviousOrQuit => {
                let current = self.entries.pop();
                match self.entries.pop() {
                    Some(concrete) => concrete,
                    None if current
                        .as_ref()
                        .is_some_and(|current| current.scope.has_tests()) =>
                    {
                        // rather than quitting, we assume the user wants to "unscope"
                        ConcreteJobRef {
                            #[expect(
                                clippy::missing_panics_doc,
                                reason = "checked in match guard"
                            )]
                            name_or_alias: current.unwrap().name_or_alias,
                            scope: Scope::default(),
                        }
                    }
                    None if *job_ref == JobRef::PreviousOrQuit => {
                        return Ok(None);
                    }
                    None => {
                        let Some(current) = current else {
                            error!("no current job"); // job stack was misused
                            return Ok(None);
                        };
                        current
                    }
                }
            }
            JobRef::Concrete(concrete) => concrete.clone(),
            JobRef::Scope(scope) => match self.entries.last() {
                Some(concrete) => ConcreteJobRef {
                    name_or_alias: concrete.name_or_alias.clone(),
                    scope: scope.clone(),
                },
                None => {
                    return Ok(None);
                }
            },
        };
        let job = match &concrete.name_or_alias {
            NameOrAlias::Alias(alias) => Job::from_alias(alias, settings),
            NameOrAlias::Name(name) => settings
                .jobs
                .get(name)
                .ok_or_else(|| anyhow!("job not found: {name:?}"))?
                .clone(),
        };
        if replace_current {
            self.entries.truncate(current_len.saturating_sub(1));
        }
        if self.entries.last() != Some(&concrete) {
            self.entries.push(concrete.clone());
        }
        Ok(Some((concrete, job)))
    }
}

/// Check that a job replaced with `replace_current` is skipped on back,
/// and that scoping applies to the replacing job
#[test]
fn test_replace_job() {
    let mut settings = Settings::default();
    for name in ["check", "doc-open", "doc"] {
        settings.jobs.insert(name.to_string(), Job::default());
    }
    settings.default_job = ConcreteJobRef::from_job_name("check");
    let mut stack = JobStack::default();
    let mut pick = |job_ref: JobRef, replace_current| {
        stack
            .pick_job(&job_ref, replace_current, &settings)
            .unwrap()
            .unwrap()
            .0
    };
    let name = ConcreteJobRef::from_job_name;
    let scope = Scope {
        tests: vec!["abc".to_string()],
    };
    assert_eq!(pick(JobRef::Initial, false), name("check"));
    assert_eq!(
        pick(JobRef::from_job_name("doc-open"), false),
        name("doc-open")
    );
    assert_eq!(pick(JobRef::from_job_name("doc"), true), name("doc"));
    assert_eq!(
        pick(JobRef::Scope(scope.clone()), false),
        ConcreteJobRef {
            name_or_alias: NameOrAlias::Name("doc".to_string()),
            scope,
        }
    );
    assert_eq!(pick(JobRef::Previous, false), name("doc"));
    assert_eq!(pick(JobRef::Previous, false), name("check"));
}
