use {
    super::*,
    rodio::OutputStream,
    std::{
        fmt,
        io::Cursor,
        time::Duration,
    },
    termimad::crossbeam::channel::Receiver,
};

struct Sound {
    bytes: &'static [u8],
    duration: Duration,
}

/// Get a sound by name, or the default sound if name is None
///
/// Names here are as near as possible from the file names in the
/// reources directory but without the number, syntax unconsistency and
/// redundancy. Resource file names are kept identical to their original
/// names to ease retrieval for attribution).
fn get_sound(name: Option<&str>) -> Result<Sound, SoundError> {
    let name = name.unwrap_or("store-scanner");
    let sound = match name {
        "2" => Sound {
            bytes: include_bytes!("../../resources/2-100419.mp3"),
            duration: Duration::from_millis(2000),
        },
        "90s-game-ui-6" => Sound {
            bytes: include_bytes!("../../resources/90s-game-ui-6-185099.mp3"),
            duration: Duration::from_millis(1300),
        },
        "beep-6" => Sound {
            bytes: include_bytes!("../../resources/beep-6-96243.mp3"),
            duration: Duration::from_millis(1000),
        },
        "beep-beep" => Sound {
            bytes: include_bytes!("../../resources/beep-beep-6151.mp3"),
            duration: Duration::from_millis(1200),
        },
        "beep-warning" => Sound {
            bytes: include_bytes!("../../resources/beep-warning-6387.mp3"),
            duration: Duration::from_millis(1200),
        },
        "bell-chord" => Sound {
            bytes: include_bytes!("../../resources/bell-chord1-83260.mp3"),
            duration: Duration::from_millis(1900),
        },
        "car-horn" => Sound {
            bytes: include_bytes!("../../resources/car-horn-beepsmp3-14659.mp3"),
            duration: Duration::from_millis(1700),
        },
        "convenience-store-ring" => Sound {
            bytes: include_bytes!("../../resources/conveniencestorering-96090.mp3"),
            duration: Duration::from_millis(1700),
        },
        "cow-bells" => Sound {
            bytes: include_bytes!("../../resources/cow_bells_01-98236.mp3"),
            duration: Duration::from_millis(1400),
        },
        "pickup" => Sound {
            bytes: include_bytes!("../../resources/pickup-sound-46472.mp3"),
            duration: Duration::from_millis(500),
        },
        "positive-beeps" => Sound {
            bytes: include_bytes!("../../resources/positive_beeps-85504.mp3"),
            duration: Duration::from_millis(600),
        },
        "short-beep-tone" => Sound {
            bytes: include_bytes!("../../resources/short-beep-tone-47916.mp3"),
            duration: Duration::from_millis(400),
        },
        "slash" => Sound {
            bytes: include_bytes!("../../resources/slash1-94367.mp3"),
            duration: Duration::from_millis(800),
        },
        "store-scanner" => Sound {
            bytes: include_bytes!("../../resources/store-scanner-beep-90395.mp3"),
            duration: Duration::from_millis(250),
        },
        "success" => Sound {
            bytes: include_bytes!("../../resources/success-48018.mp3"),
            duration: Duration::from_millis(2000),
        },
        _ => {
            return Err(SoundError::UnknownSoundName(name.to_string()));
        }
    };
    Ok(sound)
}

#[derive(Debug)]
pub enum SoundError {
    Interrupted,
    UnknownSoundName(String),
    RodioStream(rodio::StreamError),
    RodioPlay(rodio::PlayError),
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
    let sink = stream_handle.play_once(sound)?;
    sink.set_volume(psc.volume.as_part());
    if interrupt.recv_timeout(duration).is_ok() {
        info!("sound interrupted");
        Err(SoundError::Interrupted)
    } else {
        Ok(())
    }
}
