use {
    crate::*,
    anyhow::{
        anyhow,
        Context,
        Result,
    },
    std::{
        process::{
            ExitStatus,
            Stdio,
        },
        thread,
    },
    tokio::{
        io::{
            AsyncBufReadExt,
            AsyncRead,
            BufReader,
        },
        process::{
            Child,
            Command,
        },
        sync::{
            mpsc::{
                channel,
                Sender,
            },
            oneshot,
        },
        task::JoinHandle,
    },
};

/// an executor calling a cargo (or similar) command in a separate
/// thread when asked to and sending the lines of output in a channel,
/// and finishing by None.
/// Channel sizes are designed to avoid useless computations.
pub struct Executor {
    pub line_receiver: crossbeam::channel::Receiver<CommandExecInfo>,
    task_sender: Sender<Task>,
    stop_sender: oneshot::Sender<()>, // signal for stopping the thread
    thread: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Task {
    pub backtrace: bool,
}

type LineSender = crossbeam::channel::Sender<CommandExecInfo>;

impl Executor {
    /// launch the commands, send the lines of its stderr on the
    /// line channel.
    /// If `with_stdout` capture and send also its stdout.
    pub fn new(mission: &Mission) -> Result<Self> {
        let mut command = Command::from(mission.get_command());
        let with_stdout = mission.need_stdout();
        let (task_sender, mut task_receiver) = channel::<Task>(1);
        let (stop_sender, mut stop_receiver) = oneshot::channel();
        let (line_sender, line_receiver) = crossbeam::channel::unbounded();
        command
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .stdout(if with_stdout {
                Stdio::piped()
            } else {
                Stdio::null()
            });

        let thread = thread::spawn(move || {
            // start a runtime to manage the executor
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .build()
                .unwrap();

            rt.block_on(async move {
                // Handle to the current task
                let mut current_task: Option<tokio::task::JoinHandle<_>> = None;

                loop {
                    tokio::select! {
                        _ = &mut stop_receiver => break,

                        // wait for the next task
                        task = task_receiver.recv() => {
                            if let Some(old) = current_task.take() {
                                old.abort();
                            }

                            let task = match task {
                                // no more tasks, exit now
                                None => break,
                                Some(task) => task,
                            };

                            let child = match start_task(task, &mut command) {
                                Err(e) => {
                                    let response = CommandExecInfo::Error(
                                        format!("failed to start task: {}", e)
                                    );
                                    match line_sender.send(response) {
                                        Err(_) => break,
                                        _ => continue,
                                    }
                                }
                                Ok(child) => child,
                            };

                            current_task = Some(tokio::spawn(
                                execute_task(child, with_stdout, line_sender.clone())
                            ));
                        }

                        // Wait for the current task to finish
                        result = task_result(&mut current_task) => {
                            current_task = None;
                            let response = match result {
                                Err(e) => CommandExecInfo::Error(
                                    format!("failed to execute task: {}", e)
                                ),
                                Ok(status) => CommandExecInfo::End { status },
                            };

                            if line_sender.send(response).is_err() {
                                break
                            }
                        }
                    }
                }
            })
        });

        Ok(Self {
            line_receiver,
            task_sender,
            stop_sender,
            thread,
        })
    }
    /// notify the executor a computation is necessary
    pub fn start(
        &self,
        task: Task,
    ) -> Result<()> {
        self.task_sender.try_send(task)?;
        Ok(())
    }
    pub fn die(self) -> Result<()> {
        debug!("received kill order");
        self.stop_sender.send(()).unwrap();
        self.thread.join().unwrap();
        Ok(())
    }
}

async fn task_result(
    task: &mut Option<JoinHandle<Result<Option<ExitStatus>>>>
) -> Result<Option<ExitStatus>> {
    match task {
        Some(handle) => handle.await.unwrap(),
        None => match AlwaysPending.await {},
    }
}

/// A future that will never resolve
struct AlwaysPending;

impl std::future::Future for AlwaysPending {
    type Output = std::convert::Infallible;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        std::task::Poll::Pending
    }
}

/// Start the given task/command
fn start_task(
    task: Task,
    command: &mut Command,
) -> Result<Child> {
    command
        .kill_on_drop(true)
        .env("RUST_BACKTRACE", if task.backtrace { "1" } else { "0" })
        .spawn()
        .context("failed to launch command")
}

/// Send all lines in the process' output
async fn execute_task(
    mut child: Child,
    with_stdout: bool,
    line_sender: LineSender,
) -> Result<Option<ExitStatus>> {
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("child missing stderr"))?;

    let stderr_sender = line_sender.clone();
    let stderr = stream_consumer(stderr, CommandStream::StdErr, stderr_sender);

    let stdout = if with_stdout {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("child missing stdout"))?;
        let stdout_sender = line_sender.clone();
        Some(stream_consumer(
            stdout,
            CommandStream::StdOut,
            stdout_sender,
        ))
    } else {
        None
    };

    // either we wait on both stdout and stderr concurrently, or just stderr.
    if let Some(stdout) = stdout {
        tokio::try_join!(stdout, stderr)?;
    } else {
        stderr.await?;
    }

    let status = match child.wait().await {
        Err(_) => None,
        Ok(status) => Some(status),
    };

    Ok(status)
}

/// Send all lines in the given stream to the sender.
async fn stream_consumer(
    stream: impl AsyncRead + Unpin,
    origin: CommandStream,
    line_sender: LineSender,
) -> Result<()> {
    let mut lines = BufReader::new(stream).lines();

    while let Some(line) = lines.next_line().await? {
        let response = CommandExecInfo::Line(CommandOutputLine {
            content: TLine::from_tty(&line),
            origin,
        });
        if line_sender.send(response).is_err() {
            return Err(anyhow!("channel closed"));
        }
    }

    Ok(())
}
