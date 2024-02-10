use {
    crate::*,
    crokey::*,
    serde::Deserialize,
    std::collections::{
        hash_map,
        HashMap,
    },
};

/// A mapping from key combinations to actions.
///
/// Several key combinations can go to the same action.
#[derive(Debug, Clone, Deserialize)]
pub struct KeyBindings {
    #[serde(flatten)]
    map: HashMap<KeyCombination, Action>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut bindings = Self {
            map: HashMap::default(),
        };
        bindings.set(key!('?'), Internal::Help);
        bindings.set(key!(h), Internal::Help);
        bindings.set(key!(ctrl - c), Internal::Quit);
        bindings.set(key!(ctrl - q), Internal::Quit);
        bindings.set(key!(q), Internal::Quit);
        bindings.set(key!(F5), Internal::Refresh);
        bindings.set(key!(s), Internal::ToggleSummary);
        bindings.set(key!(w), Internal::ToggleWrap);
        bindings.set(key!(b), Internal::ToggleBacktrace);
        bindings.set(key!(Home), Internal::Scroll(ScrollCommand::Top));
        bindings.set(key!(End), Internal::Scroll(ScrollCommand::Bottom));
        bindings.set(key!(Up), Internal::Scroll(ScrollCommand::Lines(-1)));
        bindings.set(key!(Down), Internal::Scroll(ScrollCommand::Lines(1)));
        bindings.set(key!(PageUp), Internal::Scroll(ScrollCommand::Pages(-1)));
        bindings.set(key!(PageDown), Internal::Scroll(ScrollCommand::Pages(1)));
        bindings.set(key!(Space), Internal::Scroll(ScrollCommand::Pages(1)));
        bindings.set(key!(esc), Internal::Back);
        bindings.set(key!(ctrl - d), JobRef::Default);
        bindings.set(key!(i), JobRef::Initial);
        // keybindings for some common jobs
        bindings.set(key!(a), JobRef::from_job_name("check-all"));
        bindings.set(key!(c), JobRef::from_job_name("clippy"));
        bindings.set(key!(d), JobRef::from_job_name("doc-open"));
        bindings.set(key!(t), JobRef::from_job_name("test"));
        bindings.set(key!(r), JobRef::from_job_name("run"));
        bindings
    }
}

impl KeyBindings {
    pub fn set<A: Into<Action>>(
        &mut self,
        ck: KeyCombination,
        action: A,
    ) {
        self.map.insert(ck, action.into());
    }
    pub fn add_vim_keys(&mut self) {
        self.set(key!(g), Internal::Scroll(ScrollCommand::Top));
        self.set(key!(shift - g), Internal::Scroll(ScrollCommand::Bottom));
        self.set(key!(k), Internal::Scroll(ScrollCommand::Lines(-1)));
        self.set(key!(j), Internal::Scroll(ScrollCommand::Lines(1)));
    }
    pub fn add_all(
        &mut self,
        other: &KeyBindings,
    ) {
        for (ck, action) in other.map.iter() {
            self.map.insert(*ck, action.clone());
        }
    }
    pub fn get(
        &self,
        key: KeyCombination,
    ) -> Option<&Action> {
        self.map.get(&key)
    }
    /// return the shortest key.to_string for the internal, if any
    pub fn shortest_internal_key(
        &self,
        internal: Internal,
    ) -> Option<String> {
        let mut shortest: Option<String> = None;
        let searched_action = Action::Internal(internal);
        for (ck, action) in &self.map {
            if action == &searched_action {
                let s = ck.to_string();
                match &shortest {
                    Some(previous) if previous.len() < s.len() => {}
                    _ => {
                        shortest = Some(s);
                    }
                }
            }
        }
        shortest
    }
    /// build and return a map from actions to all the possible shortcuts
    pub fn build_reverse_map(&self) -> HashMap<&Action, Vec<KeyCombination>> {
        let mut reverse_map = HashMap::new();
        for (ck, action) in &self.map {
            reverse_map.entry(action).or_insert_with(Vec::new).push(*ck);
        }
        reverse_map
    }
}

impl<'a> IntoIterator for &'a KeyBindings {
    type Item = (&'a KeyCombination, &'a Action);
    type IntoIter = hash_map::Iter<'a, KeyCombination, Action>;
    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

#[test]
fn test_deserialize_keybindings() {
    #[derive(Deserialize)]
    struct Config {
        keybindings: KeyBindings,
    }
    let toml = r#"
    [keybindings]
    Ctrl-U = "internal:scroll-pages(-2)"
    Ctrl-d = "internal:scroll-page(1)"
    alt-q = "internal:quit"
    alt-p = "job:previous"
    "#;
    let conf = toml::from_str::<Config>(toml).unwrap();
    assert_eq!(
        conf.keybindings.get(key!(ctrl - u)),
        Some(&Action::Internal(Internal::Scroll(ScrollCommand::Pages(
            -2
        )))),
    );
    assert_eq!(
        conf.keybindings.get(key!(ctrl - d)),
        Some(&Action::Internal(Internal::Scroll(ScrollCommand::Pages(1)))),
    );
    assert_eq!(conf.keybindings.get(key!(z)), None,);
    assert_eq!(
        conf.keybindings.get(key!(alt - q)),
        Some(&Action::Internal(Internal::Quit)),
    );
    assert_eq!(
        conf.keybindings.get(key!(alt - p)),
        Some(&Action::Job(JobRef::Previous)),
    );
}
