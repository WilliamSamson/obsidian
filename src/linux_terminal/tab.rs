use gtk::{prelude::*, Box as GtkBox, Label, Orientation, Separator};

use std::{cell::RefCell, rc::Rc};

use super::{
    persist::TabSnapshot,
    profile::{profile, ProfileId},
    session::SessionView,
    settings::Settings,
};

pub(super) struct TabView {
    root: GtkBox,
    title_label: Label,
    left: SessionView,
    right: Option<SessionView>,
    split_separator: Option<Separator>,
    profile_id: ProfileId,
    settings: Rc<RefCell<Settings>>,
}

impl TabView {
    pub(super) fn new(snapshot: TabSnapshot, settings: Rc<RefCell<Settings>>) -> Self {
        let root = GtkBox::new(Orientation::Horizontal, 12);
        root.set_hexpand(true);
        root.set_vexpand(true);

        let settings_ref = settings.borrow();
        let left = SessionView::new(snapshot.profile, snapshot.left_cwd.as_deref(), &settings_ref);
        root.append(left.root());

        let mut right = None;
        let mut split_separator = None;
        if let Some(cwd) = snapshot.right_cwd.as_deref() {
            let separator = Separator::new(Orientation::Vertical);
            separator.add_css_class("obsidian-v-separator");
            root.append(&separator);
            let right_session = SessionView::new(snapshot.profile, Some(cwd), &settings_ref);
            root.append(right_session.root());
            split_separator = Some(separator);
            right = Some(right_session);
        }
        drop(settings_ref);

        let title_label = Label::new(Some(&snapshot.title));
        title_label.add_css_class("obsidian-tab-label");

        Self {
            root,
            title_label,
            left,
            right,
            split_separator,
            profile_id: snapshot.profile,
            settings,
        }
    }

    pub(super) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(super) fn title_label(&self) -> &Label {
        &self.title_label
    }

    pub(super) fn profile_id(&self) -> ProfileId {
        self.profile_id
    }

    pub(super) fn to_snapshot(&self) -> TabSnapshot {
        TabSnapshot {
            title: self.title_label.text().to_string(),
            profile: self.profile_id,
            left_cwd: self.left.current_cwd(),
            right_cwd: self.right.as_ref().and_then(SessionView::current_cwd),
        }
    }

    pub(super) fn cycle_profile(&mut self, next_profile: ProfileId) {
        self.profile_id = next_profile;
        self.left.apply_profile(next_profile);
        if let Some(right) = &self.right {
            right.apply_profile(next_profile);
        }
        self.title_label.set_text(&format!(
            "{} ({})",
            base_title(&self.title_label.text()),
            profile(next_profile).label
        ));
    }

    pub(super) fn toggle_split(&mut self) {
        if self.right.is_some() {
            if let Some(separator) = self.split_separator.take() {
                self.root.remove(&separator);
            }
            if let Some(right) = self.right.take() {
                self.root.remove(right.root());
            }
            return;
        }

        let separator = Separator::new(Orientation::Vertical);
        separator.add_css_class("obsidian-v-separator");
        self.root.append(&separator);

        let cwd = self.left.current_cwd();
        let settings_ref = self.settings.borrow();
        let right = SessionView::new(self.profile_id, cwd.as_deref(), &settings_ref);
        self.root.append(right.root());
        self.split_separator = Some(separator);
        self.right = Some(right);
    }

    pub(super) fn apply_settings(&self, settings: &Settings) {
        self.left.apply_settings(settings, self.profile_id);
        if let Some(right) = &self.right {
            right.apply_settings(settings, self.profile_id);
        }
    }
}

fn base_title(title: &str) -> &str {
    title.split(" (").next().unwrap_or(title)
}
