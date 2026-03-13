use gtk::{
    gio, prelude::*, Box as GtkBox, Orientation,
};
use vte4::{prelude::*, Terminal};

use super::{
    input,
    profile::{profile, ProfileId},
    shell,
    terminal,
};

pub(super) struct SessionView {
    root: GtkBox,
    terminal: Terminal,
}

impl SessionView {
    pub(super) fn new(profile_id: ProfileId, cwd: Option<&str>) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 8);
        root.set_hexpand(true);
        root.set_vexpand(true);

        let terminal = terminal::build_terminal(profile_id);
        let runtime = shell::spawn_shell(&terminal, cwd);

        root.append(&terminal);
        let _ = input::append_input_row(&root, &terminal, runtime.status_path());

        Self {
            root,
            terminal,
        }
    }

    pub(super) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(super) fn current_cwd(&self) -> Option<String> {
        self.terminal
            .current_directory_uri()
            .as_deref()
            .and_then(|uri| gio::File::for_uri(uri).path())
            .map(|path| path.display().to_string())
    }

    pub(super) fn apply_profile(&self, profile_id: ProfileId) {
        let config = profile(profile_id);
        self.terminal.set_font_scale(config.font_scale);
    }
}
