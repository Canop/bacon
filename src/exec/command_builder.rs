use std::{
    collections::HashMap,
    ffi::{
        OsStr,
        OsString,
    },
    path::{
        Path,
        PathBuf,
    },
    process::{
        Command,
        Stdio,
    },
};

#[derive(Debug, Clone)]
pub struct CommandBuilder {
    exe: String,
    current_dir: Option<PathBuf>,
    args: Vec<OsString>,
    with_stdout: bool,
    envs: HashMap<OsString, OsString>,
}

impl CommandBuilder {
    pub fn new(exe: &str) -> Self {
        Self {
            exe: exe.to_string(),
            current_dir: None,
            args: Vec::new(),
            with_stdout: false,
            envs: Default::default(),
        }
    }
    pub fn build(&self) -> Command {
        let mut command = Command::new(&self.exe);
        if let Some(dir) = &self.current_dir {
            command.current_dir(dir);
        }
        command.args(&self.args);
        command.envs(&self.envs);
        command
            .envs(&self.envs)
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .stdout(if self.with_stdout {
                Stdio::piped()
            } else {
                Stdio::null()
            });
        command
    }
    pub fn with_stdout(
        &mut self,
        b: bool,
    ) -> &mut Self {
        self.with_stdout = b;
        self
    }
    pub fn is_with_stdout(&self) -> bool {
        self.with_stdout
    }
    pub fn current_dir<P: AsRef<Path>>(
        &mut self,
        dir: P,
    ) -> &mut Self {
        self.current_dir = Some(dir.as_ref().to_path_buf());
        self
    }
    pub fn arg<S: AsRef<OsStr>>(
        &mut self,
        arg: S,
    ) -> &mut Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }
    pub fn args<I, S>(
        &mut self,
        args: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for arg in args {
            self.args.push(arg.as_ref().to_os_string());
        }
        self
    }
    pub fn env<K, V>(
        &mut self,
        key: K,
        val: V,
    ) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.envs
            .insert(key.as_ref().to_os_string(), val.as_ref().to_os_string());
        self
    }
    pub fn envs<I, K, V>(
        &mut self,
        vars: I,
    ) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        for (k, v) in vars {
            self.envs
                .insert(k.as_ref().to_os_string(), v.as_ref().to_os_string());
        }
        self
    }
}
