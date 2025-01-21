#[cfg(not(feature = "sound"))]
mod no_sound;
#[cfg(feature = "sound")]
mod play_sound;
mod sound_config;
#[cfg(feature = "sound")]
mod sound_player;
mod volume;

#[cfg(not(feature = "sound"))]
pub use no_sound::*;
#[cfg(feature = "sound")]
pub use {
    play_sound::*,
    sound_player::*,
};
pub use {
    sound_config::*,
    volume::*,
};

/// A command to play a sound
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PlaySoundCommand {
    pub name: Option<String>,
    pub volume: Volume,
}
