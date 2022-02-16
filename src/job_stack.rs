use {
    crate::*,
    anyhow::{anyhow, Result},
};

/// The stack of jobs that bacon ran, allowing
/// to get back to the previous one
pub struct JobStack<'c> {
    package_config: &'c PackageConfig,
    settings: &'c Settings,
    name_stack: Vec<String>,
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

    fn initial_job_name(&self) -> String {
        self.settings.arg_job_name.as_ref()
            .unwrap_or(&self.package_config.default_job)
            .to_string()
    }

    pub fn pick_job(&mut self, job_ref: &JobRef) -> Result<Option<(String, Job)>> {
        let job_name = match job_ref {
            JobRef::Default => self.package_config.default_job.to_string(),
            JobRef::Initial => self.initial_job_name(),
            JobRef::Previous => {
                self.name_stack.pop();
                match self.name_stack.pop() {
                    Some(name) => name,
                    None => {
                        return Ok(None);
                    }
                }
            }
            JobRef::Name(name) => name.to_string(),
        };
        let job = self.package_config.jobs
            .get(&job_name)
            .ok_or_else(|| anyhow!("job not found: {:?}", job_name))?;
        if self.name_stack.last() != Some(&job_name) {
            self.name_stack.push(job_name.clone());
        }
        Ok(Some((job_name, job.clone())))
    }
}
