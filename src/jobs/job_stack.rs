use {
    crate::*,
    anyhow::anyhow,
};

/// The stack of jobs that bacon ran, allowing to get back to the previous one,
/// or to scope the current one
#[derive(Default)]
pub struct JobStack {
    entries: Vec<ConcreteJobRef>,
}

/// Possible variants:
/// `Ok(job)` -> application should run the given job
/// `Err(None)` -> application should exit
/// `Err(Some(err))` -> application should exit with error
pub type JobResult = Result<(ConcreteJobRef, Job), Option<anyhow::Error>>;

impl JobStack {
    /// Apply the job ref instruction to determine the job to run, updating the stack.
    ///
    /// See [`JobResult`] for meanings of return values.
    pub fn pick_job(
        &mut self,
        job_ref: &JobRef,
        settings: &Settings,
    ) -> JobResult {
        debug!("picking job {job_ref:?}");
        let concrete = match job_ref {
            JobRef::Default => settings.default_job.clone(),
            JobRef::Initial => settings
                .arg_job
                .as_ref()
                .unwrap_or(&settings.default_job)
                .clone(),
            JobRef::PreviousOrQuit | JobRef::Previous => {
                let current = self.entries.pop();
                match self.entries.pop() {
                    Some(concrete) => concrete,
                    None if current
                        .as_ref()
                        .is_some_and(|current| current.scope.has_tests()) =>
                    {
                        // rather than quitting, we assume the user wants to "unscope"
                        ConcreteJobRef {
                            name_or_alias: current.unwrap().name_or_alias,
                            scope: Scope::default(),
                        }
                    }
                    None => {
                        if let JobRef::PreviousOrQuit = job_ref {
                            return Err(None);
                        }

                        // this has a side effect of reloading the job, which isn't ideal
                        // but the main loop needs some mission to avoid spinning forever
                        ConcreteJobRef {
                            name_or_alias: current.unwrap().name_or_alias,
                            scope: Scope::default(),
                        }
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
                    return Err(None);
                }
            },
        };
        let job = match &concrete.name_or_alias {
            NameOrAlias::Alias(alias) => Job::from_alias(alias, settings),
            NameOrAlias::Name(name) => settings
                .jobs
                .get(name)
                .ok_or_else(|| anyhow!("job not found: {:?}", name))?
                .clone(),
        };
        if self.entries.last() != Some(&concrete) {
            self.entries.push(concrete.clone());
        }
        Ok((concrete, job))
    }
}
