use crate::*;

pub type ActionMenu = Menu<Action>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionMenuDefinition {
    pub intro: Option<String>,
    pub actions: Vec<Action>,
}

impl ActionMenu {
    pub fn add_action(
        &mut self,
        action: Action,
    ) {
        self.add_item(action, None); // TODO look for key combination in settings
    }
    pub fn with_all_jobs(mission: &Mission) -> Self {
        let mut menu = Self::new();
        let mut job_names = mission.settings.jobs.keys().collect::<Vec<_>>();
        job_names.sort();
        let actions = job_names
            .into_iter()
            .map(|job_name| Action::Job(ConcreteJobRef::from_job_name(job_name).into()));
        for action in actions {
            let key = mission.settings.keybindings.shortest_key_for(&action);
            menu.add_item(action, key);
        }
        menu
    }
    pub fn from_definition(
        ActionMenuDefinition { intro, actions }: ActionMenuDefinition,
        settings: &Settings,
    ) -> Self {
        let mut menu = Self::new();
        if let Some(intro) = intro {
            menu.set_intro(intro);
        }
        for action in actions {
            let key = settings.keybindings.shortest_key_for(&action);
            menu.add_item(action, key);
        }
        menu
    }
}
