use {
    crate::*,
    anyhow::*,
    crossbeam::channel::{bounded, unbounded, Receiver, Sender},
    std::{
        io::{BufRead, BufReader},
        path::PathBuf,
        process::Stdio,
        thread,
    },
};

/// an executor calling `cargo watch` in a separate
/// thread when asked to and sending the lines of
/// output in a channel, and finishing by None.
///
/// Channel sizes are designed to avoid useless
/// computations.
pub struct Executor {
    pub task_sender: Sender<()>,
    pub line_receiver: Receiver<Result<Option<String>, String>>,
}

impl Executor {
    pub fn new(root_dir: PathBuf, use_clippy: bool) -> Result<Self> {
        let (task_sender, task_receiver) = bounded(0);
        let (line_sender, line_receiver) = unbounded();
        // TODO: interrupt and forget current computation on receiving a task ?
        thread::spawn(move || {
            for _ in task_receiver {
                let child = Report::get_command(&root_dir, use_clippy)
                    .stderr(Stdio::piped())
                    .spawn();
                let mut child = match child {
                    Ok(child) => child,
                    Err(e) => {
                        line_sender.send(Err(format!("command launch failed: {}", e))).unwrap();
                        continue;
                    }
                };
                let stderr = match child.stderr.take() {
                    Some(stderr) => stderr,
                    None => {
                        line_sender.send(Err("taking stderr failed".to_string())).unwrap();
                        continue;
                    }
                };
                for line in BufReader::new(stderr).lines() {
                    let line = line
                        .map_err(|e| e.to_string())
                        .map(|l| Some(l));
                    if let Err(e) = line_sender.send(line) {
                        debug!("error when sending line: {}", e);
                        break;
                    }
                }
                line_sender.send(Ok(None)).unwrap(); // <- "I finished" signal
                debug!("finished command execution");
            }
            debug!("closing computer");
        });
        Ok(Self {
            task_sender,
            line_receiver,
        })
    }
}
