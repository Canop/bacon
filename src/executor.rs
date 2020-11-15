use {
    crate::*,
    anyhow::*,
    crossbeam::channel::{bounded, unbounded, select, Receiver, Sender},
    std::{
        io::{BufRead, BufReader},
        process::Stdio,
        thread,
    },
};

/// an executor calling `cargo watch` in a separate
/// thread when asked to and sending the lines of
/// output in a channel, and finishing by None.
///
/// Channel sizes are designed to avoid useless computations.
pub struct Executor {
    pub line_receiver: Receiver<Result<Option<CommandOutputLine>, String>>,
    task_sender: Sender<()>,
    stop_sender: Sender<()>, // signal for stopping the thread
    thread: thread::JoinHandle<()>,
}

impl Executor {
    /// launch the commands, sends the lines of its stderr on the
    /// line channel.
    /// If `with_stdout` captures and send also its stdout.
    pub fn new(mission: &Mission) -> Result<Self> {
        let mut command = mission.get_command();
        let with_stdout = mission.need_stdout();
        let (task_sender, task_receiver) = bounded(1);
        let (stop_sender, stop_receiver) = bounded(0);
        let (line_sender, line_receiver) = unbounded();
        let thread = thread::spawn(move || {
            loop {
                select! {
                    recv(task_receiver) -> _ => {
                        debug!("starting task");
                        let mut command = command.stderr(Stdio::piped());
                        if with_stdout {
                            command = command.stdout(Stdio::piped());
                        }
                        let child = command.spawn();
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
                        // if we need stdout, we listen for in on a separate thread
                        // (if somebody knows of an efficient and clean cross-platform way
                        // to listen for both stderr and stdout on the same thread, please tell me)
                        let out_thread = if with_stdout {
                            let stdout = match child.stdout.take() {
                                Some(stdout) => stdout,
                                None => {
                                    line_sender.send(Err("taking stdout failed".to_string())).unwrap();
                                    continue;
                                }
                            };
                            let line_sender = line_sender.clone();
                            Some(thread::spawn(move|| {
                                for line in BufReader::new(stdout).lines() {
                                    let line = line
                                        .map_err(|e| e.to_string())
                                        .map(|l| {
                                            Some(CommandOutputLine {
                                                origin: CommandStream::StdOut,
                                                content: TLine::from_tty(&l),
                                            })
                                        });
                                    if let Err(e) = line_sender.send(line) {
                                        debug!("error when sending stdout line: {}", e);
                                        break;
                                    }
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
                                Err(e) => Err(e.to_string()),
                                Ok(0) => {
                                    // finished
                                    break;
                                }
                                Ok(_) => {
                                    Ok(Some(CommandOutputLine {
                                        origin: CommandStream::StdErr,
                                        content: TLine::from_tty(buf.trim_end()),
                                    }))
                                }
                            };
                            if let Err(e) = line_sender.send(r) {
                                debug!("error when sending stderr line: {}", e);
                                break;
                            }
                            buf.clear();
                        }
                        line_sender.send(Ok(None)).unwrap(); // <- "I finished" signal
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
            task_sender,
            stop_sender,
            line_receiver,
            thread,
        })
    }
    /// notify the executor a computation is necessary
    pub fn start(&self) -> Result<()> {
        self.task_sender.try_send(())?;
        Ok(())
    }
    pub fn die(self) -> Result<()> {
        debug!("received kill order");
        self.stop_sender.send(()).unwrap();
        self.thread.join().unwrap();
        Ok(())
    }
}
