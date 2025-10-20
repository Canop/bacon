use {
    crate::*,
    rustc_hash::FxHashSet,
    std::fmt,
};

#[derive(Default, Debug)]
pub struct Filter {
    dismissals: Vec<Dismissal>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Dismissal {
    Location(String),
    DiagType(String),
}

impl Dismissal {
    /// Return true if the item must be dismissed
    pub fn applies_to(
        &self,
        item: Item<'_>,
    ) -> bool {
        match self {
            Self::Location(v) => item.location().is_some_and(|loc| loc == v),
            Self::DiagType(v) => item.diag_type().is_some_and(|loc| loc == v),
        }
    }
    pub fn undo_action(&self) -> Action {
        match self {
            Self::Location(v) => Action::UndismissLocation(v.clone()),
            Self::DiagType(v) => Action::UndismissDiagType(v.clone()),
        }
    }
}

impl Filter {
    pub fn add(
        &mut self,
        dismissal: Dismissal,
    ) {
        if !self.dismissals.contains(&dismissal) {
            self.dismissals.push(dismissal);
        }
    }
    pub fn remove(
        &mut self,
        dismissal: &Dismissal,
    ) {
        self.dismissals.retain(|d| d != dismissal);
    }
    pub fn remove_dismissed_lines(
        &self,
        report: &mut Report,
    ) -> bool {
        let mut dismissed_item_idxs = FxHashSet::default();
        for item in Item::items_of(&report.lines) {
            for dismissal in &self.dismissals {
                if dismissal.applies_to(item) {
                    dismissed_item_idxs.insert(item.item_idx());
                    break;
                }
            }
        }
        let mut kept_lines = Vec::new();
        for line in report.lines.drain(..) {
            if dismissed_item_idxs.contains(&line.item_idx) {
                report.dismissed_lines.push(line);
            } else {
                kept_lines.push(line);
            }
        }
        report.lines = kept_lines;
        report.dismissed_items += dismissed_item_idxs.len();
        report.lines_changed();
        info!(
            "FILTERING, dismissed {} items: {:?}",
            dismissed_item_idxs.len(),
            dismissed_item_idxs
        );
        !dismissed_item_idxs.is_empty()
    }
    pub fn restore_dismissed_lines(
        &self,
        report: &mut Report,
    ) {
        if !report.dismissed_lines.is_empty() {
            report.lines.append(&mut report.dismissed_lines);
            report.lines.sort_by_key(|line| line.item_idx);
            report.dismissed_items = 0;
            report.lines_changed();
        }
    }
    pub fn undismiss_menu(&self) -> ActionMenu {
        let mut menu = ActionMenu::new();
        menu.add_action(Action::UndismissAll);
        for dismissal in &self.dismissals {
            menu.add_action(dismissal.undo_action());
        }
        menu
    }
}

impl fmt::Display for Dismissal {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            Self::Location(v) => write!(f, "location: {v}"),
            Self::DiagType(v) => write!(f, "diag_type: {v}"),
        }
    }
}
