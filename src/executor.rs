use {
    crate::*,
    anyhow::Result,
    crossbeam::channel::{bounded, select, unbounded, Receiver, Sender},
    std::{
        io::{BufRead, BufReader},
        process::Stdio,
        thread,
    },
};

/// an executor calling a cargo (or similar) command in a separate
/// thread when asked to and sending the lines of output in a channel,
/// and finishing by None.
/// Channel sizes are designed to avoid useless computations.
pub struct Executor {
    pub line_receiver: Receiver<CommandExecInfo>,
    task_sender: Sender<Task>,
    stop_sender: Sender<()>, // signal for stopping the thread
    thread: thread::JoinHandle<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Task {
    pub backtrace: bool,
}

impl Executor {
    /// launch the commands, sends the lines of its stderr on the
    /// line channel.
    /// If `with_stdout` captures and send also its stdout.
    pub fn new(mission: &Mission) -> Result<Self> {
        let mut command = mission.get_command();
        let with_stdout = mission.need_stdout();
        let (task_sender, task_receiver) = bounded::<Task>(1);
        let (stop_sender, stop_receiver) = bounded(0);
        let (line_sender, line_receiver) = unbounded();
        command
            .stderr(Stdio::piped())
            .stdout(if with_stdout { Stdio::piped() } else { Stdio::null() });
        let thread = thread::spawn(move || {
            loop {
                select! {
                    recv(task_receiver) -> task => {
                        let task = match task {
                            Ok(task) => task,
                            _ => { break; }
                        };
                        debug!("starting task {:?}", task);
                        command.env(
                            "RUST_BACKTRACE",
                            if task.backtrace { "1" } else { "0" },
                        );
                        let child = command.spawn();
                        let mut child = match child {
                            Ok(child) => child,
                            Err(e) => {
                                if let Err(e) = line_sender.send(CommandExecInfo::Error(
                                    format!("command launch failed: {}", e)
                                )) {
                                    debug!("error when sending launch error: {}", e);
                                    break;
                                }
                                continue;
                            }
                        };
                        let stderr = match child.stderr.take() {
                            Some(stderr) => stderr,
                            None => {
                                if let Err(e) = line_sender.send(CommandExecInfo::Error(
                                    "taking stderr failed".to_string()
                                )) {
                                    debug!("error when sending stderr error: {}", e);
                                    break;
                                }
                                continue;
                            }
                        };
                        // if we need stdout, we listen for in on a separate thread
                        // (if somebody knows of an efficient and clean cross-platform way
                        // to listen for both stderr and stdout on the same thread, please tell me)
                        let out_thread = if with_stdout {
                            let stdout = match child.stdout.take() {
                                Some(stdout) => stdout,
                                None => {
                                    if let Err(e) = line_sender.send(CommandExecInfo::Error(
                                        "taking stdout failed".to_string()
                                    )) {
                                        debug!("error when sending stdout error: {}", e);
                                        break;
                                    }
                                    continue;
                                }
                            };
                            let line_sender = line_sender.clone();
                            Some(thread::spawn(move|| {
                                let mut buf = String::new();
                                let mut buf_reader = BufReader::new(stdout);
                                loop {
                                    let r = match buf_reader.read_line(&mut buf) {
                                        Err(e) => CommandExecInfo::Error(e.to_string()),
                                        Ok(0) => {
                                            // finished
                                            break;
                                        }
                                        Ok(_) => {
                                            // debug!("STDOUT : {:?}", &buf);
                                            CommandExecInfo::Line(CommandOutputLine {
                                                origin: CommandStream::StdOut,
                                                content: TLine::from_tty(buf.trim_end()),
                                            })
                                        }
                                    };
                                    if let Err(e) = line_sender.send(r) {
                                        debug!("error when sending stdout line: {}", e);
                                        break;
                                    }
                                    buf.clear();
                                }
                            }))
                        } else {
                            None
                        };
                        let mut buf = String::new();
                        let mut buf_reader = BufReader::new(stderr);
                        loop {
                            if let Ok(()) = stop_receiver.try_recv() {
                                debug!("stopping during execution");
                                match child.kill() {
                                    Ok(()) => debug!("command stopped"),
                                    _ => debug!("command already stopped"),
                                }
                                return;
                            }
                            let r = match buf_reader.read_line(&mut buf) {
                                Err(e) => CommandExecInfo::Error(e.to_string()),
                                Ok(0) => {
                                    // finished
                                    break;
                                }
                                Ok(_) => {
                                    // debug!("STDERR : {:?}", &buf);
                                    CommandExecInfo::Line(CommandOutputLine {
                                        origin: CommandStream::StdErr,
                                        content: TLine::from_tty(buf.trim_end()),
                                    })
                                }
                            };
                            if let Err(e) = line_sender.send(r) {
                                debug!("error when sending stderr line: {}", e);
                                break;
                            }
                            buf.clear();
                        }
                        let status = match child.wait() {
                            Ok(exit_status) => {
                                debug!("exit_status: {:?}", &exit_status);
                                Some(exit_status)
                            }
                            Err(e) => {
                                warn!("error in child: {:?}", e);
                                None
                            }
                        };
                        if let Err(e) = line_sender.send(CommandExecInfo::End { status }) {
                            debug!("error when sending line: {}", e);
                            break;
                        }
                        debug!("finished command execution");
                        if let Some(thread) = out_thread {
                            debug!("waiting for out listening thread to join");
                            thread.join().unwrap();
                            debug!("out listening thread joined");
                        }
                    }
                    recv(stop_receiver) -> _ => {
                        debug!("leaving thread");
                        return;
                    }
                }
            }
        });
        Ok(Self {
            line_receiver,
            task_sender,
            stop_sender,
            thread,
        })
    }
    /// notify the executor a computation is necessary
    pub fn start(&self, task: Task) -> Result<()> {
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
