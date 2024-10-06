use {
    crate::*,
    anyhow::{
        Result,
        anyhow,
    },
};

/// The stack of jobs that bacon ran, allowing to get back to the previous one,
/// or to scope the current one
pub struct JobStack<'c> {
    settings: &'c Settings,
    entries: Vec<ConcreteJobRef>,
}

impl<'c> JobStack<'c> {
    pub fn new(settings: &'c Settings) -> Self {
        Self {
            settings,
            entries: Vec::new(),
        }
    }

    fn initial_job(&self) -> &ConcreteJobRef {
        self.settings
            .arg_job
            .as_ref()
            .unwrap_or(&self.settings.default_job)
    }

    /// Apply the job ref instruction to determine the job to run, updating the stack.
    ///
    /// When no job is returned, the application is supposed to quit.
    pub fn pick_job(
        &mut self,
        job_ref: &JobRef,
    ) -> Result<Option<(ConcreteJobRef, Job)>> {
        debug!("picking job {job_ref:?}");
        let concrete = match job_ref {
            JobRef::Default => self.settings.default_job.clone(),
            JobRef::Initial => self.initial_job().clone(),
            JobRef::Previous => {
                let current = self.entries.pop();
                match self.entries.pop() {
                    Some(concrete) => concrete,
                    None if current
                        .as_ref()
                        .map_or(false, |current| current.scope.has_tests()) =>
                    {
                        // rather than quitting, we assume the user wants to "unscope"
                        ConcreteJobRef {
                            name_or_alias: current.unwrap().name_or_alias,
                            scope: Scope::default(),
                        }
                    }
                    None => {
                        return Ok(None);
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
            NameOrAlias::Alias(alias) => Job::from_alias(alias, self.settings),
            NameOrAlias::Name(name) => self
                .settings
                .jobs
                .get(name)
                .ok_or_else(|| anyhow!("job not found: {:?}", name))?
                .clone(),
        };
        if self.entries.last() != Some(&concrete) {
            self.entries.push(concrete.clone());
        }
        Ok(Some((concrete, job)))
    }
}
