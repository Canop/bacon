use {
    super::*,
    std::thread,
    termimad::crossbeam::channel::{
        self,
        Sender,
    },
};

/// An instruction for the beeper
#[derive(Debug, Clone)]
enum Instruction {
    PlaySound(PlaySoundCommand),
    Die,
}

/// Manage a thread to play sounds without blocking bacon
pub struct SoundPlayer {
    thread: Option<thread::JoinHandle<()>>,
    sender: Option<Sender<Instruction>>,
}
impl SoundPlayer {
    pub fn new(base_volume: Volume) -> anyhow::Result<Self> {
        let (sender, receiver) = channel::bounded(1);
        let thread = thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(Instruction::PlaySound(mut ps)) => {
                        ps.volume = ps.volume * base_volume;
                        if let Err(e) = play_sound(&ps) {
                            error!("sound error: {}", e);
                        }
                    }
                    Ok(Instruction::Die) => {
                        info!("sound player thread is stopping");
                        break;
                    }
                    Err(e) => {
                        error!("sound player channel error: {}", e);
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
            let _ = sender.try_send(Instruction::PlaySound(PlaySoundCommand::default()));
        }
    }
    /// Requests a sound, unless there's already one in the queue
    /// (we don't want to stack sounds)
    pub fn play(
        &self,
        beep: PlaySoundCommand,
    ) {
        if let Some(sender) = &self.sender {
            info!("sending beep command");
            let _ = sender.try_send(Instruction::PlaySound(beep));
        }
    }
    /// Make the beeper thread synchronously stop
    /// (wait for the current sound to end)
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
                info!("SoundPlayer gracefully stopped");
            }
        }
    }
}
impl Drop for SoundPlayer {
    fn drop(&mut self) {
        self.die();
    }
}
