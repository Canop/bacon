use {
    crate::*,
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

/// A "concrete" job ref is one which can be used from the start, without
/// referring to the job stack
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConcreteJobRef {
    pub name_or_alias: NameOrAlias,
    pub scope: Scope,
}

impl ConcreteJobRef {
    pub fn from_job_name<S: Into<String>>(s: S) -> Self {
        Self {
            name_or_alias: NameOrAlias::Name(s.into()),
            scope: Default::default(),
        }
    }
    pub fn badge_label(&self) -> String {
        let mut s = String::new();
        match &self.name_or_alias {
            NameOrAlias::Name(name) => {
                s.push_str(name);
            }
            NameOrAlias::Alias(alias) => {
                s.push_str(alias);
            }
        }
        if self.scope.has_tests() {
            s.push_str(" (scoped)");
        }
        s
    }
    pub fn with_scope(
        mut self,
        scope: Scope,
    ) -> Self {
        self.scope = scope;
        self
    }
}

impl Default for ConcreteJobRef {
    fn default() -> Self {
        Self {
            name_or_alias: NameOrAlias::Name("check".to_string()),
            scope: Default::default(),
        }
    }
}

impl fmt::Display for ConcreteJobRef {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match &self.name_or_alias {
            NameOrAlias::Alias(alias) => write!(f, "alias:{alias}")?,
            NameOrAlias::Name(name) => write!(f, "{name}")?,
        }
        if self.scope.has_tests() {
            write!(f, "({})", self.scope.tests.join(","))?;
        }
        Ok(())
    }
}
impl FromStr for ConcreteJobRef {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("empty job name")
        } else {
            Ok(s.into())
        }
    }
}
impl Serialize for ConcreteJobRef {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for ConcreteJobRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

impl From<&str> for ConcreteJobRef {
    fn from(str_entry: &str) -> Self {
        let Some((_, alias_prefix, name_or_alias, scope)) =
            regex_captures!(r"^(alias:)?([^\(\)]+)(?:\(([^\)]+)\))?$", str_entry,)
        else {
            warn!("unexpected job ref: {str_entry:?}");
            return Self::from_job_name(str_entry.to_string());
        };
        let name_or_alias = if alias_prefix.is_empty() {
            NameOrAlias::Name(name_or_alias.to_string())
        } else {
            NameOrAlias::Alias(name_or_alias.to_string())
        };
        let scope = Scope {
            tests: scope
                .split(',')
                .filter(|t| !t.trim().is_empty())
                .map(|s| s.to_string())
                .collect(),
        };
        Self {
            name_or_alias,
            scope,
        }
    }
}

impl JsonSchema for ConcreteJobRef {
    fn schema_name() -> Cow<'static, str> {
        "ConcreteJobRef".into()
    }
    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::ConcreteJobRef").into()
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
