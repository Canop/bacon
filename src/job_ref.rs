use {
    crate::*,
    lazy_regex::*,
    serde::{
        Deserialize,
        Deserializer,
        de,
    },
    std::{
        fmt,
        str::FromStr,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JobRef {
    Default,
    Initial,
    Previous,
    Concrete(ConcreteJobRef),
    Scope(Scope),
}

impl JobRef {
    pub fn from_job_name<S: Into<String>>(s: S) -> Self {
        Self::Concrete(ConcreteJobRef::from_job_name(s))
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NameOrAlias {
    Name(String),
    Alias(String),
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
            warn!("unexpected job ref: {:?}", str_entry);
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

impl fmt::Display for JobRef {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::Initial => write!(f, "initial"),
            Self::Previous => write!(f, "previous"),
            Self::Scope(Scope { tests }) => write!(f, "scope:{}", tests.join(",")),
            Self::Concrete(concrete) => write!(f, "{}", concrete),
        }
    }
}

impl From<&str> for JobRef {
    fn from(s: &str) -> Self {
        regex_switch!(s,
            "^default$"i => Self::Default,
            "^default$"i => Self::Default,
            "^initial$"i => Self::Initial,
            "^previous$"i => Self::Previous,
            "^scope:(?<tests>.+)$" => Self::Scope(Scope {
                tests: tests
                    .split(',')
                    .filter(|t| !t.trim().is_empty())
                    .map(|s| s.to_string())
                    .collect(),
            }),
        )
        .unwrap_or_else(|| Self::Concrete(ConcreteJobRef::from(s)))
    }
}

#[test]
fn test_job_ref_string_round_trip() {
    let job_refs = vec![
        JobRef::Default,
        JobRef::Initial,
        JobRef::Previous,
        JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Name("run".to_string()),
            scope: Scope::default(),
        }),
        JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Name("nextest".to_string()),
            scope: Scope {
                tests: vec!["first::test".to_string(), "second_test".to_string()],
            },
        }),
        JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Alias("my-check".to_string()),
            scope: Scope::default(),
        }),
        JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Alias("my-test".to_string()),
            scope: Scope {
                tests: vec!["abc".to_string()],
            },
        }),
        JobRef::Concrete(ConcreteJobRef {
            name_or_alias: NameOrAlias::Name("nextest".to_string()),
            scope: Scope {
                tests: vec!["abc".to_string()],
            },
        }),
        JobRef::Scope(Scope {
            tests: vec!["first::test".to_string(), "second_test".to_string()],
        }),
    ];
    for job_ref in job_refs {
        let s = job_ref.to_string();
        let job_ref2 = JobRef::from(s.as_str());
        assert_eq!(job_ref, job_ref2);
    }
}
