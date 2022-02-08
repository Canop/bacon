
pub enum JobRef {
    Default,
    Name(String),
}

impl JobRef {
    pub fn from_app_arg(arg: &Option<String>) -> Self {
        match arg {
            Some(job_name) => Self::Name(job_name.clone()),
            None => Self::Default,
        }
    }
    pub fn from_internal(name: &str) -> Self {
        if name == "default" {
            Self::Default
        } else {
            Self::Name(name.to_string())
        }
    }
}
