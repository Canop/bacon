use {
    schemars::{
        JsonSchema,
        Schema,
        SchemaGenerator,
        json_schema,
    },
    serde::{
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
        de,
    },
    std::{
        borrow::Cow,
        fmt,
        ops::Mul,
        str::FromStr,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Volume {
    /// Volume in [0, 100]
    percent: u16, // u16 ensures we can multiply it without overflow
}
impl Default for Volume {
    fn default() -> Self {
        Self { percent: 100 }
    }
}
impl Mul for Volume {
    type Output = Self;
    fn mul(
        self,
        rhs: Self,
    ) -> Self {
        Self {
            percent: self.percent * rhs.percent / 100,
        }
    }
}
impl Volume {
    pub fn new(percent: u16) -> Self {
        Self {
            percent: percent.clamp(0, 100),
        }
    }
    /// Return the volume in [0, 100]
    pub fn as_percent(self) -> u16 {
        self.percent
    }
    /// Return the volume in [0, 1]
    pub fn as_part(self) -> f32 {
        f32::from(self.percent) / 100f32
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum ParseVolumeError {
    ValueOutOfRange,
    NotU16(String),
}
impl fmt::Display for ParseVolumeError {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::ValueOutOfRange => write!(f, "value out of [0-100] range"),
            Self::NotU16(s) => write!(f, "value '{s}' is not a valid integer"),
        }
    }
}
impl std::error::Error for ParseVolumeError {}

impl FromStr for Volume {
    type Err = ParseVolumeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_end_matches('%');
        let percent: u16 = s
            .parse()
            .map_err(|_| ParseVolumeError::NotU16(s.to_string()))?;
        if percent > 100 {
            return Err(ParseVolumeError::ValueOutOfRange);
        }
        Ok(Self { percent })
    }
}
impl std::fmt::Display for Volume {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}%", self.percent)
    }
}
impl Serialize for Volume {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}
impl<'de> Deserialize<'de> for Volume {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(de::Error::custom)
    }
}
impl JsonSchema for Volume {
    fn schema_name() -> Cow<'static, str> {
        "Volume".into()
    }
    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::Volume").into()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
        })
    }
    fn inline_schema() -> bool {
        true
    }
}
