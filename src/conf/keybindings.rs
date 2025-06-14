use {
    crate::*,
    crokey::*,
    serde::Deserialize,
    std::{
        collections::{
            HashMap,
            hash_map,
        },
        fmt,
    },
};

/// A mapping from key combinations to actions.
///
/// Several key combinations can go to the same action.
#[derive(Clone, Deserialize)]
pub struct KeyBindings {
    #[serde(flatten)]
    map: HashMap<KeyCombination, Action>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        let mut bindings = Self {
            map: HashMap::default(),
        };
        bindings.set(key!('?'), Action::Help);
        bindings.set(key!(h), Action::Help);
        bindings.set(key!(ctrl - c), Action::Quit);
        bindings.set(key!(ctrl - q), Action::Quit);
        bindings.set(key!(q), Action::Quit);
        bindings.set(key!(F5), Action::Refresh);
        bindings.set(key!(s), Action::ToggleSummary);
        bindings.set(key!(w), Action::ToggleWrap);
        bindings.set(key!(b), Action::ToggleBacktrace("1"));
        bindings.set(key!(Home), Action::Scroll(ScrollCommand::Top));
        bindings.set(key!(End), Action::Scroll(ScrollCommand::Bottom));
        bindings.set(key!(Up), Action::Scroll(ScrollCommand::Lines(-1)));
        bindings.set(key!(Down), Action::Scroll(ScrollCommand::Lines(1)));
        bindings.set(key!(PageUp), Action::Scroll(ScrollCommand::pages(-1)));
        bindings.set(key!(PageDown), Action::Scroll(ScrollCommand::pages(1)));
        bindings.set(key!(Space), Action::Scroll(ScrollCommand::MilliPages(800)));
        bindings.set(key!(f), Action::ScopeToFailures);
        bindings.set(key!(esc), Action::Back);
        bindings.set(key!(ctrl - d), JobRef::Default);
        bindings.set(key!(i), JobRef::Initial);
        bindings.set(key!(p), Action::TogglePause);
        bindings.set(key!('/'), Action::FocusSearch);
        bindings.set(key!(':'), Action::FocusGoto);
        bindings.set(key!(enter), Action::Validate);
        bindings.set(key!(tab), Action::NextMatch);
        bindings.set(key!(backtab), Action::PreviousMatch);
        bindings.set(key!(shift - backtab), Action::PreviousMatch);
        bindings.set(key!(ctrl - j), Action::OpenJobsMenu);

        // keybindings for some common jobs
        bindings.set(key!(a), JobRef::from_job_name("check-all"));
        bindings.set(key!(c), JobRef::from_job_name("clippy"));
        bindings.set(key!(d), JobRef::from_job_name("doc-open"));
        bindings.set(key!(t), JobRef::from_job_name("test"));
        bindings.set(key!(n), JobRef::from_job_name("nextest"));
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
        self.set(key!(g), Action::Scroll(ScrollCommand::Top));
        self.set(key!(shift - g), Action::Scroll(ScrollCommand::Bottom));
        self.set(key!(k), Action::Scroll(ScrollCommand::Lines(-1)));
        self.set(key!(j), Action::Scroll(ScrollCommand::Lines(1)));
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
    /// return the shortest key.to_string for the action, if any
    pub fn shortest_key_for(
        &self,
        action: &Action,
    ) -> Option<KeyCombination> {
        self.shortest_key(|a| a == action)
    }
    /// return the key combination for the action matching the filter, choosing
    /// the one with the shortest Display representation.
    pub fn shortest_key<F>(
        &self,
        filter: F,
    ) -> Option<KeyCombination>
    where
        F: Fn(&Action) -> bool,
    {
        let mut shortest: Option<(KeyCombination, String)> = None;
        for (&ck, action) in &self.map {
            if filter(action) {
                let s = ck.to_string();
                match &shortest {
                    Some(previous) if previous.1.len() < s.len() => {}
                    _ => {
                        shortest = Some((ck, s));
                    }
                }
            }
        }
        shortest.map(|o| o.0)
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

impl fmt::Debug for KeyBindings {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut ds = f.debug_struct("KeyBindings");
        for (kc, action) in &self.map {
            ds.field(&kc.to_string(), &action.to_string());
        }
        ds.finish()
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
    Ctrl-U = "internal:scroll-pages(-.5)"
    Ctrl-d = "internal:scroll-page(1)"
    alt-q = "internal:quit"
    ctrl-q = "quit"
    alt-p = "job:previous"
    ctrl-p = "play-sound"
    ctrl-shift-p = "play-sound(volume=100)"
    "#;
    let conf = toml::from_str::<Config>(toml).unwrap();
    assert_eq!(
        conf.keybindings.get(key!(ctrl - u)),
        Some(&Action::Scroll(ScrollCommand::MilliPages(-500))),
    );
    assert_eq!(
        conf.keybindings.get(key!(ctrl - d)),
        Some(&Action::Scroll(ScrollCommand::MilliPages(1000))),
    );
    assert_eq!(conf.keybindings.get(key!(z)), None,);
    assert_eq!(conf.keybindings.get(key!(alt - q)), Some(&Action::Quit),);
    assert_eq!(
        conf.keybindings.get(key!(alt - p)),
        Some(&Action::Job(JobRef::Previous)),
    );
}
