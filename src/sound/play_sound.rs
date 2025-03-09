use {
    super::*,
    rodio::{OutputStream, Decoder, Source},
    std::{
        fmt,
        collections::HashMap,
        io::Cursor,
        sync::RwLock,
        time::Duration,
    },
    termimad::crossbeam::channel::Receiver,
    once_cell::sync::Lazy as LazyCell,
};

#[derive(Clone, Copy)]
struct Sound {
    bytes: &'static [u8],
    duration: Duration,
}

static SOUNDS: LazyCell<RwLock<HashMap<&'static str, Sound>>> = LazyCell::new(|| {
    let mut sounds = HashMap::new();
    #[cfg(feature = "default-sounds")]
    sounds.extend(DEFAULT_SOUNDS);
    RwLock::new(sounds)
});

#[cfg(feature = "default-sounds")]
const DEFAULT_SOUNDS: [(&str, Sound); 15] = [
    ("2", Sound {
        bytes: include_bytes!("../../resources/2-100419.mp3"),
        duration: Duration::from_millis(2000),
    }),
    ("90s-game-ui-6", Sound {
        bytes: include_bytes!("../../resources/90s-game-ui-6-185099.mp3"),
        duration: Duration::from_millis(1300),
    }),
    ("beep-6", Sound {
        bytes: include_bytes!("../../resources/beep-6-96243.mp3"),
        duration: Duration::from_millis(1000),
    }),
    ("beep-beep", Sound {
        bytes: include_bytes!("../../resources/beep-beep-6151.mp3"),
        duration: Duration::from_millis(1200),
    }),
    ("beep-warning", Sound {
        bytes: include_bytes!("../../resources/beep-warning-6387.mp3"),
        duration: Duration::from_millis(1200),
    }),
    ("bell-chord", Sound {
        bytes: include_bytes!("../../resources/bell-chord1-83260.mp3"),
        duration: Duration::from_millis(1900),
    }),
    ("car-horn", Sound {
        bytes: include_bytes!("../../resources/car-horn-beepsmp3-14659.mp3"),
        duration: Duration::from_millis(1700),
    }),
    ("convenience-store-ring", Sound {
        bytes: include_bytes!("../../resources/conveniencestorering-96090.mp3"),
        duration: Duration::from_millis(1700),
    }),
    ("cow-bells", Sound {
        bytes: include_bytes!("../../resources/cow_bells_01-98236.mp3"),
        duration: Duration::from_millis(1400),
    }),
    ("pickup", Sound {
        bytes: include_bytes!("../../resources/pickup-sound-46472.mp3"),
        duration: Duration::from_millis(500),
    }),
    ("positive-beeps", Sound {
        bytes: include_bytes!("../../resources/positive_beeps-85504.mp3"),
        duration: Duration::from_millis(600),
    }),
    ("short-beep-tone", Sound {
        bytes: include_bytes!("../../resources/short-beep-tone-47916.mp3"),
        duration: Duration::from_millis(400),
    }),
    ("slash", Sound {
        bytes: include_bytes!("../../resources/slash1-94367.mp3"),
        duration: Duration::from_millis(800),
    }),
    ("store-scanner", Sound {
        bytes: include_bytes!("../../resources/store-scanner-beep-90395.mp3"),
        duration: Duration::from_millis(250),
    }),
    ("success", Sound {
        bytes: include_bytes!("../../resources/success-48018.mp3"),
        duration: Duration::from_millis(2000),
    }),
];

/// Get a sound by name; or the default sound if name is None,
/// and the default-sounds feature is enabled.
///
/// There are too kinds of sounds: default and custom.
/// 
/// Names here are as near as possible from the file names in the
/// reources directory but without the number, syntax unconsistency and
/// redundancy. Resource file names are kept identical to their original
/// names to ease retrival for attribution).
fn get_sound(name: Option<&str>) -> Result<Sound, SoundError> {
    // NOTE: This doesn't distinguish from whether the default-sound feature is
    // enabled, and might confuse users. But then again, that only happens when
    // a default name is requested while default-sound feature is disabled,
    // which shouldn't happen anyway.
    let name = name.unwrap_or("store-scanner");
    SOUNDS.read().unwrap().get(name).copied().ok_or_else(|| SoundError::UnknownSoundName(name.to_string()))
}

/// Add a sound from a file (path).
/// 
/// Bytes are leaked, as they are loaded once and kept throughout the program's
/// lifetime, just like the default sounds.
/// Likewise, the name is also leaked.
pub(crate) fn add_sound(name: &str, path: &str) -> Result<(), SoundError> {
    // ideally we should accept AsRef<Path>, but `shellexpand::tilde` requires
    // a AsRef<str>. To ease things, allow only allow UTF-8 paths.
    let path = shellexpand::tilde(path).to_string();
    let bytes: &'static [u8] = std::fs::read(&path)
        .map_err(|_| SoundError::MissingSoundFile(path.to_string()))?
        .leak();
    // TODO: check for duration right here, or use Option?
    // If to check, might as well replace `bytes` with decoded struct
    SOUNDS.write().unwrap()
        .insert(name.to_string().leak(), Sound {
            bytes,
            duration: Duration::ZERO,
        });
    info!("loaded sound {name:?} from {path:?}");
    Ok(())
}

#[derive(Debug)]
pub enum SoundError {
    Interrupted,
    UnknownSoundName(String),
    MissingSoundFile(String),
    RodioDecode(rodio::decoder::DecoderError),
    RodioStream(rodio::StreamError),
    RodioPlay(rodio::PlayError),
}
impl From<rodio::decoder::DecoderError> for SoundError {
    fn from(e: rodio::decoder::DecoderError) -> Self {
        SoundError::RodioDecode(e)
    }
}
impl From<rodio::StreamError> for SoundError {
    fn from(e: rodio::StreamError) -> Self {
        SoundError::RodioStream(e)
    }
}
impl From<rodio::PlayError> for SoundError {
    fn from(e: rodio::PlayError) -> Self {
        SoundError::RodioPlay(e)
    }
}
impl fmt::Display for SoundError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            SoundError::Interrupted => write!(f, "sound interrupted"),
            SoundError::UnknownSoundName(name) => write!(f, "unknown sound name: {}", name),
            SoundError::MissingSoundFile(path) => write!(f, "missing sound file: {}", path),
            SoundError::RodioDecode(e) => write!(f, "rodio decode error: {}", e),
            SoundError::RodioStream(e) => write!(f, "rodio stream error: {}", e),
            SoundError::RodioPlay(e) => write!(f, "rodio play error: {}", e),
        }
    }
}

impl std::error::Error for SoundError {}

/// Play the requested sound, sleeps for its duration (until interrupted)
pub fn play_sound(
    psc: &PlaySoundCommand,
    interrupt: Receiver<()>,
) -> Result<(), SoundError> {
    debug!("play sound: {:#?}", psc);
    let Sound { bytes, duration } = get_sound(psc.name.as_deref())?;
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sound = Cursor::new(bytes);
    let decoder = Decoder::new(sound.clone())?;
    let duration = if duration == Duration::ZERO {
        decoder.total_duration()
    } else {
        Some(duration)
    };
    let sink = stream_handle.play_once(sound)?;
    sink.set_volume(psc.volume.as_part());
    if duration.is_some() && interrupt.recv_timeout(duration.unwrap()).is_ok() {
        info!("sound interrupted");
        Err(SoundError::Interrupted)
    } else {
        Ok(())
    }
}
