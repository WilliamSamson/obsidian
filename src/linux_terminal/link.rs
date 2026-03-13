use std::{cell::RefCell, rc::Rc};

use gtk::{gio, prelude::*, Button, EventControllerMotion, GestureClick};
use vte4::{prelude::*, Regex, Terminal};

#[derive(Clone, Copy)]
pub(super) struct LinkMatcher {
    path_tag: i32,
}

impl LinkMatcher {
    pub(super) fn install(terminal: &Terminal) -> Option<Self> {
        let regex = Regex::for_match(r"(~|/)[^\s:]+", 0).ok()?;
        let path_tag = terminal.match_add_regex(&regex, 0);
        terminal.match_set_cursor_name(path_tag, "pointer");
        Some(Self { path_tag })
    }

    fn uri_at_position(&self, terminal: &Terminal, x: f64, y: f64) -> Option<String> {
        if let Some(uri) = terminal.check_hyperlink_at(x, y) {
            return Some(uri.to_string());
        }

        let (matched, tag) = terminal.check_match_at(x, y);
        if tag != self.path_tag {
            return None;
        }

        matched.map(|path| path_to_uri(terminal, path.as_str()))
    }
}

pub(super) fn wire_open_actions(terminal: &Terminal, open_button: &Button) {
    let Some(matcher) = LinkMatcher::install(terminal) else {
        return;
    };

    // `Rc<RefCell<_>>` lets the motion tracker and open button share the hovered link on the GTK UI thread.
    let hovered_uri = Rc::new(RefCell::new(None::<String>));
    let motion = EventControllerMotion::new();
    let terminal_for_motion = terminal.clone();
    let hovered_uri_for_motion = hovered_uri.clone();
    motion.connect_motion(move |_, x, y| {
        *hovered_uri_for_motion.borrow_mut() =
            matcher.uri_at_position(&terminal_for_motion, x, y);
    });
    terminal.add_controller(motion);

    let terminal_for_click = terminal.clone();
    let click = GestureClick::new();
    click.connect_released(move |gesture, _, x, y| {
        let modifiers = gesture.current_event_state();
        let is_ctrl_click = modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK);
        if !is_ctrl_click {
            return;
        }

        if let Some(uri) = matcher.uri_at_position(&terminal_for_click, x, y) {
            let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>);
        }
    });
    terminal.add_controller(click);

    open_button.connect_clicked(move |_| {
        if let Some(uri) = hovered_uri.borrow().clone() {
            let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>);
        }
    });
}

fn path_to_uri(terminal: &Terminal, path: &str) -> String {
    if path.starts_with("file://") || path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }

    let absolute = if let Some(stripped) = path.strip_prefix("~/") {
        std::env::var("HOME")
            .map(|home| format!("{home}/{stripped}"))
            .unwrap_or_else(|_| path.to_string())
    } else if path == "~" {
        std::env::var("HOME").unwrap_or_else(|_| path.to_string())
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        terminal
            .current_directory_uri()
            .as_deref()
            .and_then(|uri| gio::File::for_uri(uri).path())
            .map(|cwd| cwd.join(path).display().to_string())
            .unwrap_or_else(|| path.to_string())
    };

    format!("file://{absolute}")
}
