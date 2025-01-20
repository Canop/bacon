use {
    serde::{
        Deserialize,
        Deserializer,
        Serialize,
        Serializer,
        de,
    },
    std::{
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
        self.percent as f32 / 100f32
    }
}
impl FromStr for Volume {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_end_matches('%');
        let percent: u16 = s.parse().map_err(|_| "number in 0-100 expected")?;
        Ok(Self {
            percent: percent.clamp(0, 100),
        })
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
