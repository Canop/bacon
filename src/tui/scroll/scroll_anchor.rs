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
        str::FromStr,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAnchor {
    Auto, // not the same as a None: it can override a None
    First,
    Last,
}

impl fmt::Display for ScrollAnchor {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            ScrollAnchor::Auto => write!(f, "auto"),
            ScrollAnchor::First => write!(f, "first"),
            ScrollAnchor::Last => write!(f, "last"),
        }
    }
}

impl FromStr for ScrollAnchor {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(ScrollAnchor::Auto),
            "first" => Ok(ScrollAnchor::First),
            "last" => Ok(ScrollAnchor::Last),
            _ => Err(format!("invalid scroll anchor: {}", s)),
        }
    }
}
impl Serialize for ScrollAnchor {
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
impl<'de> Deserialize<'de> for ScrollAnchor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl JsonSchema for ScrollAnchor {
    fn schema_name() -> Cow<'static, str> {
        "ScrollAnchor".into()
    }
    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::ScrollAnchor").into()
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
