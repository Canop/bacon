use {
    crate::*,
    schemars::{
        JsonSchema,
        Schema,
        SchemaGenerator,
        json_schema,
    },
    serde::{
        Deserialize,
        Deserializer,
        de,
    },
    std::{
        borrow::Cow,
        fmt,
        path::{
            Path,
            PathBuf,
        },
        time::Duration,
    },
};
#[cfg(feature = "sound")]
use {
    rodio::OutputStreamBuilder,
    std::{
        io::Cursor,
        time::Instant,
    },
    termimad::crossbeam::channel::{
        Receiver,
        Select,
    },
};

/// How often a sound being played is checked for its end
#[cfg(feature = "sound")]
const CHECK_PERIOD: Duration = Duration::from_millis(50);

/// A sound, either embedded in the bacon executable or given by the
/// path of an audio file, with an optional duration after which
/// playing is cut.
///
/// In configuration, a sound is either the path of its file, or a table
/// with a path and a duration, eg
///
/// ```TOML
/// [sounds]
/// bepop = "~/audio/bepop.mp3"
/// success = { path = "~/audio/tada.ogg", duration = "1500ms" }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sound {
    source: SoundSource,
    cut: Option<Duration>,
}

#[derive(Clone, PartialEq, Eq)]
enum SoundSource {
    Embedded(&'static [u8]),
    File(PathBuf),
}

impl Sound {
    /// Build a sound from bytes embedded in the executable, cut after the
    /// given duration
    pub const fn embedded(
        bytes: &'static [u8],
        cut_millis: u64,
    ) -> Self {
        Self {
            source: SoundSource::Embedded(bytes),
            cut: Some(Duration::from_millis(cut_millis)),
        }
    }
    /// Duration after which the sound is cut, if any
    pub fn cut(&self) -> Option<Duration> {
        self.cut
    }
    /// The path of the sound file, when the sound isn't embedded
    pub fn path(&self) -> Option<&Path> {
        match &self.source {
            SoundSource::Embedded(_) => None,
            SoundSource::File(path) => Some(path),
        }
    }
    /// Expand the tilde and make the path absolute, relative paths
    /// being taken from the given directory
    pub fn resolve_path(
        &mut self,
        base_dir: &Path,
    ) {
        if let SoundSource::File(path) = &mut self.source {
            *path = resolve_path(path, base_dir);
        }
    }
}

impl fmt::Debug for SoundSource {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::Embedded(bytes) => write!(f, "embedded sound of {} bytes", bytes.len()),
            Self::File(path) => write!(f, "{}", path.display()),
        }
    }
}

#[cfg(feature = "sound")]
impl Sound {
    /// Check the sound can be read, without decoding it
    pub fn check(&self) -> Result<(), SoundError> {
        if let SoundSource::File(path) = &self.source {
            std::fs::metadata(path).map_err(|e| SoundError::Read(path.clone(), e))?;
        }
        Ok(())
    }
    /// The encoded audio data, read from the file if the sound isn't embedded
    fn bytes(&self) -> Result<Cow<'static, [u8]>, SoundError> {
        match &self.source {
            SoundSource::Embedded(bytes) => Ok(Cow::Borrowed(bytes)),
            SoundSource::File(path) => std::fs::read(path)
                .map(Cow::Owned)
                .map_err(|e| SoundError::Read(path.clone(), e)),
        }
    }
    /// Play the sound, returning when it's finished or cut, or as soon
    /// as the interrupter says it must stop
    pub fn play<M>(
        &self,
        volume: Volume,
        interrupter: &Interrupter<M>,
    ) -> Result<Option<Interruption>, SoundError> {
        let bytes = self.bytes()?;
        let stream = OutputStreamBuilder::open_default_stream()?;
        let sink = rodio::play(stream.mixer(), Cursor::new(bytes))?;
        sink.set_volume(volume.as_part());
        let end = self.cut.map(|cut| Instant::now() + cut);
        loop {
            if sink.empty() {
                return Ok(None);
            }
            let mut wait = CHECK_PERIOD;
            if let Some(end) = end {
                let Some(remaining) = end.checked_duration_since(Instant::now()) else {
                    return Ok(None);
                };
                wait = wait.min(remaining);
            }
            if let Some(interruption) = interrupter.wait(wait) {
                return Ok(Some(interruption));
            }
        }
    }
}

/// Why a sound stops before its natural end
#[cfg(feature = "sound")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interruption {
    /// another sound is waiting to be played
    NewSound,
    /// the sound player is stopping
    Die,
}

/// Tells a sound being played when it must stop
#[cfg(feature = "sound")]
pub struct Interrupter<'c, M> {
    die: &'c Receiver<()>,
    next: &'c Receiver<M>,
}

#[cfg(feature = "sound")]
impl<'c, M> Interrupter<'c, M> {
    pub fn new(
        die: &'c Receiver<()>,
        next: &'c Receiver<M>,
    ) -> Self {
        Self { die, next }
    }
    /// Wait for at most the given duration, telling what must stop the
    /// sound. Nothing is consumed, so the message is still there for
    /// the player thread.
    fn wait(
        &self,
        duration: Duration,
    ) -> Option<Interruption> {
        let mut select = Select::new();
        select.recv(self.die);
        select.recv(self.next);
        match select.ready_timeout(duration) {
            Err(_) => None,
            Ok(_) if self.die.is_empty() => Some(Interruption::NewSound),
            Ok(_) => Some(Interruption::Die),
        }
    }
}

#[cfg(feature = "sound")]
#[test]
fn test_interrupter() {
    use termimad::crossbeam::channel;
    let (s_die, r_die) = channel::bounded::<()>(1);
    let (s_next, r_next) = channel::bounded::<()>(1);
    let interrupter = Interrupter::new(&r_die, &r_next);
    let period = Duration::from_millis(5);
    assert_eq!(interrupter.wait(period), None);
    s_next.send(()).unwrap();
    assert_eq!(interrupter.wait(period), Some(Interruption::NewSound));
    s_die.send(()).unwrap();
    // dying wins over the sound waiting
    assert_eq!(interrupter.wait(period), Some(Interruption::Die));
    // and nothing was consumed
    assert_eq!(r_next.len(), 1);
    assert_eq!(r_die.len(), 1);
}

impl<'de> Deserialize<'de> for Sound {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(SoundVisitor)
    }
}

struct SoundVisitor;
impl<'de> de::Visitor<'de> for SoundVisitor {
    type Value = Sound;
    fn expecting(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            f,
            "a sound file path, or a table with a path and a duration"
        )
    }
    fn visit_str<E: de::Error>(
        self,
        path: &str,
    ) -> Result<Sound, E> {
        Ok(Sound {
            source: SoundSource::File(path.into()),
            cut: None,
        })
    }
    fn visit_map<M: de::MapAccess<'de>>(
        self,
        mut map: M,
    ) -> Result<Sound, M::Error> {
        let mut path: Option<PathBuf> = None;
        let mut cut: Option<Duration> = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_ref() {
                "path" => {
                    path = Some(map.next_value()?);
                }
                "duration" => {
                    // a zero duration (eg "none") means no cut
                    let period = map.next_value::<Period>()?;
                    cut = (!period.is_zero()).then_some(period.duration);
                }
                _ => {
                    return Err(de::Error::unknown_field(&key, &["path", "duration"]));
                }
            }
        }
        let path = path.ok_or_else(|| de::Error::missing_field("path"))?;
        Ok(Sound {
            source: SoundSource::File(path),
            cut,
        })
    }
}

impl JsonSchema for Sound {
    fn schema_name() -> Cow<'static, str> {
        "Sound".into()
    }
    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::Sound").into()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "oneOf": [
                {
                    "type": "string",
                    "description": "Path of the sound file.",
                },
                {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" },
                        "duration": { "type": "string" },
                    },
                    "required": ["path"],
                },
            ],
        })
    }
    fn inline_schema() -> bool {
        true
    }
}
