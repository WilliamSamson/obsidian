use gtk::{prelude::*, Box as GtkBox, Label, Orientation, Separator};

use super::{
    persist::TabSnapshot,
    profile::{profile, ProfileId},
    session::SessionView,
};

pub(super) struct TabView {
    root: GtkBox,
    title_label: Label,
    left: SessionView,
    right: Option<SessionView>,
    split_separator: Option<Separator>,
    profile_id: ProfileId,
}

impl TabView {
    pub(super) fn new(snapshot: TabSnapshot) -> Self {
        let root = GtkBox::new(Orientation::Horizontal, 12);
        root.set_hexpand(true);
        root.set_vexpand(true);

        let left = SessionView::new(snapshot.profile, snapshot.left_cwd.as_deref());
        root.append(left.root());

        let mut right = None;
        let mut split_separator = None;
        if let Some(cwd) = snapshot.right_cwd.as_deref() {
            let separator = Separator::new(Orientation::Vertical);
            separator.add_css_class("obsidian-v-separator");
            root.append(&separator);
            let right_session = SessionView::new(snapshot.profile, Some(cwd));
            root.append(right_session.root());
            split_separator = Some(separator);
            right = Some(right_session);
        }

        let title_label = Label::new(Some(&snapshot.title));
        title_label.add_css_class("obsidian-tab-label");

        Self {
            root,
            title_label,
            left,
            right,
            split_separator,
            profile_id: snapshot.profile,
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
        let right = SessionView::new(self.profile_id, cwd.as_deref());
        self.root.append(right.root());
        self.split_separator = Some(separator);
        self.right = Some(right);
    }
}

fn base_title(title: &str) -> &str {
    title.split(" (").next().unwrap_or(title)
}
