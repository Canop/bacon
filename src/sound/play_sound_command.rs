use {
    super::Volume,
    rodio::OutputStream,
    std::{
        io::Cursor,
        thread,
        time::Duration,
    },
};

/// A command to play a sound
///
/// There might be other settings in the future, like the sound to play
/// and the duration, dependings on whether I'm given the sound resources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PlaySoundCommand {
    pub volume: Volume,
}

impl PlaySoundCommand {
    /// Do a beep, sleeps for its duration
    ///
    /// Apparently, rodio stream things
    /// - kill the sound as soon as they're dropped
    /// - can't be reused
    ///
    /// So it's preferable not to call this directly from a working or
    /// UI thread but to use the SoundPlayer struct which manages a thread.
    pub fn play(&self) -> anyhow::Result<()> {
        debug!("beep");
        let bytes = include_bytes!("../../resources/beep.ogg");
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sound = Cursor::new(bytes);
        let sink = stream_handle.play_once(sound)?;
        sink.set_volume(self.volume.as_part());
        thread::sleep(Duration::from_millis(500));
        Ok(())
    }
}
