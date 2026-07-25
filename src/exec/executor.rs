use {
    crate::*,
    std::{
        io::{
            self,
            BufRead,
            BufReader,
        },
        process::{
            Child,
            Command,
        },
        thread,
        time::Instant,
    },
    termimad::crossbeam::channel::{
        self,
        Receiver,
        Sender,
    },
};

/// an executor calling a cargo (or similar) command in a separate
/// thread when asked to and sending the lines of output in a channel,
/// and finishing by None.
/// Channel sizes are designed to avoid useless computations.
pub struct MissionExecutor {
    command_builder: CommandBuilder,
    kill_command: Option<Vec<String>>,
    line_sender: Sender<CommandExecInfo>,
    pub line_receiver: Receiver<CommandExecInfo>,
}

/// Dedicated to one execution of the job (so there's usually
/// several task executors during the lifetime of a mission executor)
pub struct TaskExecutor {
    /// the thread running the current task
    child_thread: thread::JoinHandle<()>,
    stop_sender: Sender<StopMessage>,
    grace_period_start: Option<Instant>, // forgotten at end of grace period
    grace_period: Period,
}

/// A message sent to the `child_thread` on end
#[derive(Clone, Copy)]
enum StopMessage {
    SendStatus, // process already finished, just get status
    Kill,       // kill the process, don't bother about the status
}

impl TaskExecutor {
    /// Interrupt the process
    pub fn interrupt(self) {
        let _ = self.stop_sender.send(StopMessage::Kill);
    }
    /// Kill the process, and wait until it finished
    pub fn die(self) {
        if let Err(e) = self.stop_sender.send(StopMessage::Kill) {
            debug!("failed to send 'die' signal: {e}");
        }
        if self.child_thread.join().is_err() {
            warn!("child_thread.join() failed"); // should not happen
        }
    }
    pub fn is_in_grace_period(&mut self) -> bool {
        if let Some(grace_period_start) = self.grace_period_start {
            if grace_period_start.elapsed() < self.grace_period.duration {
                return true;
            }
            self.grace_period_start = None;
        }
        false
    }
}

impl MissionExecutor {
    /// Prepare the executor (no task/process/thread is started at this point)
    pub fn new(mission: &Mission) -> anyhow::Result<Self> {
        let command_builder = mission.get_command()?;
        let kill_command = mission.kill_command();
        let (line_sender, line_receiver) = channel::unbounded();
        Ok(Self {
            command_builder,
            kill_command,
            line_sender,
            line_receiver,
        })
    }

    /// Start the job's command, once, with the given settings
    ///
    /// # Panics
    ///
    /// Will panic if the `MissionBuilder` doesn't pipe stderr
    pub fn start(
        &mut self,
        task: Task,
    ) -> anyhow::Result<TaskExecutor> {
        info!("start task {task:?}");
        // Discard any output still queued from the previous task. Its reader
        // threads were joined by die() before we got here, so nothing new can
        // arrive during this drain. Without it, lines buffered by the previous
        // (killed) execution would leak into this one's output and report.
        while self.line_receiver.try_recv().is_ok() {}
        let grace_period = task.grace_period;
        let grace_period_start = if grace_period.is_zero() {
            None
        } else {
            Some(Instant::now())
        };
        let mut command_builder = self.command_builder.clone();
        if let Some(backtrace) = task.backtrace {
            command_builder.env("RUST_BACKTRACE", backtrace);
        }
        let kill_command = self.kill_command.clone();
        let with_stdout = command_builder.is_with_stdout();
        let line_sender = self.line_sender.clone();
        let (stop_sender, stop_receiver) = channel::bounded(1);
        let err_stop_sender = stop_sender.clone();

        // Global task executor thread
        let child_thread = thread::spawn(move || {
            // before starting the command, we wait some time, so that a bunch
            // of quasi-simultaneous file events can be finished before the command
            // starts (during this time, no other command is started by bacon in app.rs)
            if !grace_period.is_zero() {
                thread::sleep(grace_period.duration);
            }

            let mut cmd = command_builder.build();
            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    let _ = line_sender.send(CommandExecInfo::Error(
                        anyhow::anyhow!(e).context(format!("failed to spawn {cmd:?}")),
                    ));
                    return;
                }
            };

            // thread piping the stdout lines
            let stdout_thread = if with_stdout {
                match child.stdout.take() {
                    Some(stdout) => {
                        let sender = line_sender.clone();
                        let mut buf_reader = BufReader::new(stdout);
                        Some(thread::spawn(move || {
                            let mut line = String::new();
                            loop {
                                match buf_reader.read_line(&mut line) {
                                    Ok(0) => {
                                        // there won't be anything more, quitting
                                        break;
                                    }
                                    Ok(_) => {
                                        let response =
                                            CommandExecInfo::Line(RawCommandOutputLine {
                                                content: line.clone(),
                                                origin: CommandStream::StdOut,
                                            });
                                        if sender.send(response).is_err() {
                                            break; // channel closed
                                        }
                                    }
                                    Err(e) => {
                                        warn!("error reading stdout: {e}");
                                        if e.kind() != io::ErrorKind::InvalidData {
                                            // a genuine I/O error (not just a
                                            // non-UTF-8 line, whose bytes were
                                            // already consumed): stop, so we don't
                                            // spin and the join can complete
                                            break;
                                        }
                                        // InvalidData: skip this line, keep reading
                                    }
                                }
                                line.clear();
                            }
                        }))
                    }
                    None => {
                        warn!("process has no stdout"); // unlikely
                        None
                    }
                }
            } else {
                None
            };

            // starting a thread to handle stderr lines until program
            // ends (then ask the child_thread to send status)
            let err_line_sender = line_sender.clone();
            // stderr is piped in CommandBuilder, so the following statement can't fail
            // unless you break the CommandBuilder
            let stderr = child
                .stderr
                .take()
                .expect("MissionExecutor requires piped stderr");
            let mut buf_reader = BufReader::new(stderr);
            let stderr_thread = thread::spawn(move || {
                let mut line = String::new();
                loop {
                    match buf_reader.read_line(&mut line) {
                        Ok(0) => {
                            if let Err(e) = err_stop_sender.send(StopMessage::SendStatus) {
                                warn!("sending stop message failed: {e}");
                            }
                            break;
                        }
                        Ok(_) => {
                            let response = CommandExecInfo::Line(RawCommandOutputLine {
                                content: line.clone(),
                                origin: CommandStream::StdErr,
                            });
                            if err_line_sender.send(response).is_err() {
                                break; // channel closed
                            }
                        }
                        Err(e) => {
                            warn!("error reading stderr: {e}");
                            if e.kind() != io::ErrorKind::InvalidData {
                                // a genuine I/O error (not just a non-UTF-8 line):
                                // treat it as end-of-stream so child_thread doesn't
                                // wait for the status message forever, and stop
                                if let Err(e) = err_stop_sender.send(StopMessage::SendStatus) {
                                    warn!("sending stop message failed: {e}");
                                }
                                break;
                            }
                            // InvalidData: skip this line, keep reading
                        }
                    }
                    line.clear();
                }
            });

            // now waiting for the stop event
            let mut end_status = None;
            match stop_receiver.recv() {
                Ok(stop) => match stop {
                    StopMessage::SendStatus => {
                        // capture the status now, but don't announce End yet: we
                        // want all output lines flushed (readers joined) first
                        end_status = child.wait().ok();
                    }
                    StopMessage::Kill => {
                        debug!("explicit interrupt received");
                        kill(kill_command.as_deref(), &mut child);
                    }
                },
                Err(e) => {
                    debug!("recv error: {e}"); // probably just the executor dropped
                    kill(kill_command.as_deref(), &mut child);
                }
            }
            if let Err(e) = child.wait() {
                warn!("waiting for child failed: {e}");
            }
            // Wait for the reader threads to drain and send all their lines
            // before this task ends. Otherwise a line buffered by a killed
            // process could still be sent on the (mission-wide) channel after
            // die() returns, and leak into the next task's output. It also
            // guarantees every output line precedes the End message below.
            if let Some(stdout_thread) = stdout_thread {
                let _ = stdout_thread.join();
            }
            let _ = stderr_thread.join();
            // now that all output has been sent, announce completion
            if let Some(status) = end_status {
                let _ = line_sender.send(CommandExecInfo::End { status });
            }
        });
        Ok(TaskExecutor {
            child_thread,
            stop_sender,
            grace_period_start,
            grace_period,
        })
    }
}

/// kill the child process, either by using a specific command or by
/// using the default platform kill method if the specific command
/// failed or wasn't provided.
fn kill(
    kill_command: Option<&[String]>,
    child: &mut Child,
) {
    if let Some(kill_command) = kill_command {
        info!("launch specific kill command {kill_command:?}");
        let Err(e) = run_kill_command(kill_command, child) else {
            return;
        };
        warn!("specific kill command failed: {e}");
    }
    if let Err(e) = child.kill() {
        // e.g. the process already exited; nothing more we can do, and panicking
        // here would skip the child.wait() of the caller and orphan the process
        warn!("failed to kill child process: {e}");
    }
}

fn run_kill_command(
    kill_command: &[String],
    child: &mut Child,
) -> io::Result<()> {
    let (exe, args) = kill_command
        .split_first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "empty kill command"))?;
    let mut kill = Command::new(exe);
    kill.args(args);
    kill.arg(child.id().to_string());
    let mut proc = kill.spawn()?;
    let status = proc.wait()?;
    if !status.success() {
        return Err(io::Error::other(format!(
            "kill command returned nonzero status: {status}"
        )));
    }
    child.wait()?;
    Ok(())
}
