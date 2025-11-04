use {
    anyhow::anyhow,
    lazy_regex::*,
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
        str::FromStr,
        time::Duration,
    },
};

/// A small wrapper over `time::Duration`, to allow reading from a string in
/// config. There's no symmetric serialization and the input format is
/// quite crude (eg "25ms" or "254ns" or "none")
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Period {
    pub duration: Duration,
}

impl Period {
    pub const fn is_zero(&self) -> bool {
        self.duration.is_zero()
    }
    pub fn sleep(&self) {
        std::thread::sleep(self.duration);
    }
}

impl From<Duration> for Period {
    fn from(duration: Duration) -> Self {
        Self { duration }
    }
}

impl FromStr for Period {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let duration = regex_switch!(s,
            r"^(?<n>\d+)\s*ns$" => Duration::from_nanos(n.parse()?),
            r"^(?<n>\d+)\s*ms$" => Duration::from_millis(n.parse()?),
            r"^(?<n>\d+)\s*s$" => Duration::from_secs(n.parse()?),
            r"^[^1-9]*$" => Duration::new(0, 0), // eg "none", "0", "off"
        )
        .ok_or_else(|| anyhow!("Invalid period: {s}"))?;
        Ok(Self { duration })
    }
}

impl<'de> Deserialize<'de> for Period {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}
impl JsonSchema for Period {
    fn schema_name() -> Cow<'static, str> {
        "Period".into()
    }
    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::Period").into()
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "Duration expressed as a human-readable string such as \"15ms\" or \"2s\".",
        })
    }
    fn inline_schema() -> bool {
        true
    }
}
