use {
    super::*,
    std::thread,
    termimad::crossbeam::channel::{
        self,
        Sender,
        select,
    },
};

/// The maximum number of sounds that can be queued, as we don't want
/// a long list of sounds for events that are triggered in a loop
/// (on quit, the queue will be interrupted anyway).
const MAX_QUEUED_SOUNDS: usize = 2;

/// Manage a thread to play sounds without blocking bacon
pub struct SoundPlayer {
    thread: Option<thread::JoinHandle<()>>,
    s_die: Option<Sender<()>>,
    s_sound: Sender<PlaySoundCommand>,
}
impl SoundPlayer {
    pub fn new(base_volume: Volume) -> anyhow::Result<Self> {
        let (s_sound, r_sound) = channel::bounded::<PlaySoundCommand>(MAX_QUEUED_SOUNDS);
        let (s_die, r_die) = channel::bounded(1);
        let thread = thread::spawn(move || {
            loop {
                select! {
                    recv(r_die) -> _ => {
                        info!("sound player thread is stopping");
                        break;
                    }
                    recv(r_sound) -> ps => {
                        match ps {
                            Ok(mut ps) => {
                                if !r_die.is_empty() {
                                    continue;
                                }
                                ps.volume = ps.volume * base_volume;
                                match play_sound(&ps, r_die.clone()) {
                                    Ok(()) => {
                                        debug!("sound played");
                                    }
                                    Err(SoundError::Interrupted) => {
                                        // only reason is sound player is dying
                                        info!("sound interrupted");
                                        break;
                                    }
                                    Err(e) => {
                                        error!("sound error: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("sound player channel error: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });
        Ok(Self {
            thread: Some(thread),
            s_die: Some(s_die),
            s_sound,
        })
    }
    /// Requests a sound, unless too many of them are already queued
    pub fn play(
        &self,
        sound_command: PlaySoundCommand,
    ) {
        if self.s_sound.try_send(sound_command).is_err() {
            warn!("Too many sounds in the queue, dropping one");
        }
    }
    /// Make the beeper thread synchronously stop
    /// (interrupting the current sound if any)
    pub fn die(&mut self) {
        if let Some(sender) = self.s_die.take() {
            if let Err(e) = sender.send(()) {
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
