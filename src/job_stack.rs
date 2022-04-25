use {
    crate::*,
    anyhow::{anyhow, Result},
};

/// The stack of jobs that bacon ran, allowing
/// to get back to the previous one
pub struct JobStack<'c> {
    package_config: &'c PackageConfig,
    settings: &'c Settings,
    name_stack: Vec<JobType>,
}

impl<'c> JobStack<'c> {
    pub fn new(
        package_config: &'c PackageConfig,
        settings: &'c Settings,
    ) -> Self {
        Self {
            package_config,
            settings,
            name_stack: Vec::new(),
        }
    }

    fn initial_job(&self) -> JobType {
        self.settings.arg_job_name.as_ref().map_or_else(
            || JobType::Job(self.package_config.default_job.clone()),
            Clone::clone,
        )
    }

    pub fn pick_job(&mut self, job_ref: &JobRef) -> Result<Option<(JobType, Job)>> {
        let job_type = match job_ref {
            JobRef::Default => JobType::Job(self.package_config.default_job.to_string()),
            JobRef::Initial => self.initial_job(),
            JobRef::Previous => {
                self.name_stack.pop();
                match self.name_stack.pop() {
                    Some(name) => name,
                    None => {
                        return Ok(None);
                    }
                }
            }
            JobRef::Type(ty) => ty.clone(),
        };

        if self.name_stack.last() != Some(&job_type) {
            self.name_stack.push(job_type.clone());
        }
        let job = match &job_type {
            JobType::Job(job_name) => self
                .package_config
                .jobs
                .get(job_name)
                .ok_or_else(|| anyhow!("job not found: {:?}", job_name))?
                .clone(),
            JobType::Alias(alias_name) => Job::from_alias(alias_name, self.settings),
        };
        Ok(Some((job_type, job)))
    }
}
