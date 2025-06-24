use super::*;

/// A dummy sound player which does nothing
pub struct SoundPlayer {}
impl SoundPlayer {
    pub fn new(_base_volume: Volume) -> anyhow::Result<Self> {
        Err(anyhow::anyhow!(
            "Bacon is compiled without the sound feature"
        ))
    }
    pub fn play(
        &self,
        _beep: PlaySoundCommand,
    ) {
        // should never be called as the  sound player is not instantiated
    }
}
