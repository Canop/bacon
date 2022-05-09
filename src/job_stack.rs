use {
    crate::*,
    anyhow::{anyhow, Result},
};

/// The stack of jobs that bacon ran, allowing
/// to get back to the previous one
pub struct JobStack<'c> {
    package_config: &'c PackageConfig,
    settings: &'c Settings,
    entries: Vec<ConcreteJobRef>,
}

impl<'c> JobStack<'c> {

    pub fn new(
        package_config: &'c PackageConfig,
        settings: &'c Settings,
    ) -> Self {
        Self {
            package_config,
            settings,
            entries: Vec::new(),
        }
    }

    fn initial_job(&self) -> &ConcreteJobRef {
        self.settings.arg_job.as_ref()
            .unwrap_or(&self.package_config.default_job)
    }


    pub fn pick_job(&mut self, job_ref: &JobRef) -> Result<Option<(ConcreteJobRef, Job)>> {
        info!("PICKING JOB {job_ref:?}");
        let concrete = match job_ref {
            JobRef::Default => self.package_config.default_job.clone(),
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
            ConcreteJobRef::Alias(alias) => Job::from_alias(alias, &self.settings),
            ConcreteJobRef::Name(name) => {
                self.package_config.jobs
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
