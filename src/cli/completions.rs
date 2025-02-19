use {
    clap_complete::CompletionCandidate,
    std::process::Command,
};

fn with_self_command(
    f: impl FnOnce(Command) -> Option<Vec<CompletionCandidate>>
) -> Vec<CompletionCandidate> {
    std::env::current_exe()
        .ok()
        .and_then(|command| f(Command::new(command)))
        .unwrap_or_default()
}

pub fn list_jobs() -> Vec<CompletionCandidate> {
    with_self_command(|mut c| {
        let output = c.arg("--completion-list-jobs").output().ok()?;
        let output: String = String::from_utf8(output.stdout).ok()?;
        Some(output.split('\0').map(CompletionCandidate::new).collect())
    })
}
