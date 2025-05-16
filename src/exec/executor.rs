use {
    crate::*,
    std::{
        collections::HashSet,
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

/// A message sent to the child_thread on end
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
    pub fn new(
        mission: &Mission,
        options: &HashSet<String>,
    ) -> anyhow::Result<Self> {
        let command_builder = mission.get_command(options)?;
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
    pub fn start(
        &mut self,
        task: Task,
    ) -> anyhow::Result<TaskExecutor> {
        info!("start task {task:?}");
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
            if with_stdout {
                let sender = line_sender.clone();
                let Some(stdout) = child.stdout.take() else {
                    warn!("process has no stdout"); // unlikely
                    return;
                };
                let mut buf_reader = BufReader::new(stdout);
                thread::spawn(move || {
                    let mut line = String::new();
                    loop {
                        match buf_reader.read_line(&mut line) {
                            Err(e) => {
                                warn!("error : {e}");
                            }
                            Ok(0) => {
                                // there won't be anything more, quitting
                                break;
                            }
                            Ok(_) => {
                                let response = CommandExecInfo::Line(RawCommandOutputLine {
                                    content: line.clone(),
                                    origin: CommandStream::StdOut,
                                });
                                if sender.send(response).is_err() {
                                    break; // channel closed
                                }
                            }
                        }
                        line.clear();
                    }
                });
            }

            // starting a thread to handle stderr lines until program
            // ends (then ask the child_thread to send status)
            let err_line_sender = line_sender.clone();
            let stderr = child.stderr.take().expect("child missing stderr");
            let mut buf_reader = BufReader::new(stderr);
            thread::spawn(move || {
                let mut line = String::new();
                loop {
                    match buf_reader.read_line(&mut line) {
                        Err(e) => {
                            warn!("error : {e}");
                        }
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
                    }
                    line.clear();
                }
            });

            // now waiting for the stop event
            match stop_receiver.recv() {
                Ok(stop) => match stop {
                    StopMessage::SendStatus => {
                        let status = child.try_wait();
                        if let Ok(status) = status {
                            let _ = line_sender.send(CommandExecInfo::End { status });
                        }
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
    child.kill().expect("command couldn't be killed")
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
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("kill command returned nonzero status: {status}"),
        ));
    }
    child.wait()?;
    Ok(())
}
