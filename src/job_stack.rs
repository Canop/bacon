use {
    crate::*,
    anyhow::{anyhow, Result},
};

/// The stack of jobs that bacon ran, allowing
/// to get back to the previous one
pub struct JobStack<'c> {
    settings: &'c Settings,
    entries: Vec<ConcreteJobRef>,
}

impl<'c> JobStack<'c> {

    pub fn new(
        settings: &'c Settings,
    ) -> Self {
        Self {
            settings,
            entries: Vec::new(),
        }
    }

    fn initial_job(&self) -> &ConcreteJobRef {
        self.settings.arg_job.as_ref()
            .unwrap_or(&self.settings.default_job)
    }

    pub fn pick_job(&mut self, job_ref: &JobRef) -> Result<Option<(ConcreteJobRef, Job)>> {
        debug!("picking job {job_ref:?}");
        let concrete = match job_ref {
            JobRef::Default => self.settings.default_job.clone(),
            JobRef::Initial => self.initial_job().clone(),
            JobRef::Previous => {
                self.entries.pop();
                match self.entries.pop() {
                    Some(concrete) => concrete,
                    None => {
                        return Ok(None);
                    }
                }
            }
            JobRef::Concrete(concrete) => concrete.clone(),
        };
        let job = match &concrete {
            ConcreteJobRef::Alias(alias) => Job::from_alias(alias, self.settings),
            ConcreteJobRef::Name(name) => {
                self.settings.jobs
                    .get(name)
                    .ok_or_else(|| anyhow!("job not found: {:?}", name))?
                    .clone()
            }
        };
        if self.entries.last() != Some(&concrete) {
            self.entries.push(concrete.clone());
        }
        Ok(Some((concrete, job)))
    }
}
