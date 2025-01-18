use {
    rodio::{
        Decoder,
        OutputStream,
        source::Source,
    },
    std::{
        io::Cursor,
        thread,
        time::Duration,
    },
    termimad::crossbeam::channel::{
        self,
        Sender,
    },
};

/// Do a beep, but sleeps for its duration
///
/// Apparently, rodio stream things
/// - kill the sound as soon as they're dropped
/// - can't be reused
/// So it's preferable not to call this directly from a working or
/// UI thread but to use the Beeper struct which manages a thread.
pub fn beep() -> anyhow::Result<()> {
    debug!("beep");
    let bytes = include_bytes!("../resources/beep.ogg");
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sound = Cursor::new(bytes);
    let source = Decoder::new(sound)?;
    stream_handle.play_raw(source.convert_samples())?;
    thread::sleep(Duration::from_millis(500));
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum Instruction {
    Beep,
    Die,
}

pub struct Beeper {
    thread: Option<thread::JoinHandle<()>>,
    sender: Option<Sender<Instruction>>,
}
impl Beeper {
    pub fn new() -> anyhow::Result<Self> {
        let (sender, receiver) = channel::bounded(1);
        let thread = thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(Instruction::Beep) => {
                        if let Err(e) = beep() {
                            error!("beep error: {}", e);
                        }
                    }
                    Ok(Instruction::Die) => {
                        info!("beeper thread is stopping");
                        break;
                    }
                    Err(e) => {
                        error!("beeper channel error: {}", e);
                        break;
                    }
                }
            }
        });
        Ok(Self {
            thread: Some(thread),
            sender: Some(sender),
        })
    }
    /// Requests a beep, unless there's already one in the queue
    /// (we don't want to beep too much)
    pub fn beep(&self) {
        if let Some(sender) = &self.sender {
            info!("sending beep signal");
            let _ = sender.try_send(Instruction::Beep);
        }
    }
    /// Make the beeper thread synchronously stop
    /// (wait for the current beep to end)
    pub fn die(&mut self) {
        if let Some(sender) = self.sender.take() {
            if let Err(e) = sender.send(Instruction::Die) {
                warn!("failed to send 'kill' signal: {e}");
            }
        }
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                warn!("child_thread.join() failed"); // should not happen
            } else {
                info!("Beeper gracefully stopped");
            }
        }
    }
}
impl Drop for Beeper {
    fn drop(&mut self) {
        self.die();
    }
}
