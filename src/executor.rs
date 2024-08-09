use {
    crate::*,
    anyhow::{
        Context,
        Result,
    },
    crossbeam::channel::{
        Receiver,
        Sender,
    },
    std::{
        io::{
            BufRead,
            BufReader,
        },
        process::{
            Child,
            Command,
            Stdio,
        },
        thread,
    },
};

/// an executor calling a cargo (or similar) command in a separate
/// thread when asked to and sending the lines of output in a channel,
/// and finishing by None.
/// Channel sizes are designed to avoid useless computations.
pub struct MissionExecutor {
    command: Command,
    kill_command: Vec<String>,
    /// whether it's necessary to transmit stdout lines
    with_stdout: bool,
    line_sender: Sender<CommandExecInfo>,
    pub line_receiver: Receiver<CommandExecInfo>,
}

pub struct TaskExecutor {
    /// the thread running the current task
    child_thread: thread::JoinHandle<()>,
    stop_sender: Sender<StopMessage>,
}

/// A message sent to the child_thread on end
#[derive(Clone, Copy)]
enum StopMessage {
    SendStatus, // process already finished, just get status
    Kill,       // kill the process, don't bother about the status
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Task {
    pub backtrace: bool,
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
}

impl MissionExecutor {
    /// Prepare the executor (no task/process/thread is started at this point)
    pub fn new(mission: &Mission) -> Result<Self> {
        let mut command = mission.get_command();
        let kill_command = mission.kill_command();
        let with_stdout = mission.need_stdout();
        let (line_sender, line_receiver) = crossbeam::channel::unbounded();
        command
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .stdout(if with_stdout {
                Stdio::piped()
            } else {
                Stdio::null()
            });
        Ok(Self {
            command,
            kill_command,
            with_stdout,
            line_sender,
            line_receiver,
        })
    }

    pub fn start(
        &mut self,
        task: Task,
    ) -> Result<TaskExecutor> {
        info!("start task {task:?}");
        let mut child = self
            .command
            .env("RUST_BACKTRACE", if task.backtrace { "1" } else { "0" })
            .spawn()
            .context("failed to launch command")?;
        let kill_command = self.kill_command.clone();
        let with_stdout = self.with_stdout;
        let line_sender = self.line_sender.clone();
        let (stop_sender, stop_receiver) = crossbeam::channel::bounded(1);
        let err_stop_sender = stop_sender.clone();

        // Global task executor thread
        let child_thread = thread::spawn(move || {
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
                                let response = CommandExecInfo::Line(CommandOutputLine {
                                    content: TLine::from_tty(&line),
                                    origin: CommandStream::StdErr,
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
                            let response = CommandExecInfo::Line(CommandOutputLine {
                                content: TLine::from_tty(&line),
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
                        kill(&kill_command, &mut child).unwrap_or_else({
                            || child.kill().expect("command couldn't be killed")
                        });
                    }
                },
                Err(e) => {
                    debug!("recv error: {e}"); // probably just the executor dropped
                    child.kill().expect("command couldn't be killed");
                }
            }
            if let Err(e) = child.wait() {
                warn!("waiting for child failed: {e}");
            }
        });
        //self.line_receiver = line_receiver;
        Ok(TaskExecutor {
            child_thread,
            stop_sender,
        })
    }
}

fn kill(
    kill_command: &Vec<String>,
    child: &mut Child,
) -> Option<()> {
    let (exe, args) = kill_command.split_first()?;
    let mut kill = Command::new(exe);
    kill.args(args);
    kill.arg(child.id().to_string());
    let mut proc = kill
        .spawn()
        .map_err(|e| {
            warn!("could not kill child: {e}");
            e
        })
        .ok()?;
    let status = proc
        .wait()
        .map_err(|e| {
            warn!("command could not be killed: {e}");
            e
        })
        .ok()?;
    if !status.success() {
        warn!("kill command returned nonzero status: {status}");
        return None;
    }
    child.wait().ok();
    Some(())
}
