use {
    crate::*,
    lazy_regex::*,
    std::fmt,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JobRef {
    Default,
    Initial,
    Previous,
    PreviousOrQuit,
    Concrete(ConcreteJobRef),
    Scope(Scope),
}

impl JobRef {
    pub fn from_job_name<S: Into<String>>(s: S) -> Self {
        Self::Concrete(ConcreteJobRef::from_job_name(s))
    }
}

impl From<Scope> for JobRef {
    fn from(scope: Scope) -> Self {
        Self::Scope(scope)
    }
}

impl From<ConcreteJobRef> for JobRef {
    fn from(concrete: ConcreteJobRef) -> Self {
        Self::Concrete(concrete)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NameOrAlias {
    Name(String),
    Alias(String),
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
            Self::PreviousOrQuit => write!(f, "previous-or-quit"),
            Self::Scope(Scope { tests }) => write!(f, "scope:{}", tests.join(",")),
            Self::Concrete(concrete) => write!(f, "{concrete}"),
        }
    }
}

impl From<&str> for JobRef {
    fn from(s: &str) -> Self {
        regex_switch!(s,
            "^default$"i => Self::Default,
            "^initial$"i => Self::Initial,
            "^previous$"i => Self::Previous,
            "^previous-or-quit$"i => Self::PreviousOrQuit,
            "^scope:(?<tests>.+)$"i => Self::Scope(Scope {
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
        JobRef::PreviousOrQuit,
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
