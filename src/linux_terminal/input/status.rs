use std::{cell::Cell, fs, path::Path};

use gtk::{glib, prelude::*, Label};

pub(super) fn build_status_label(status_path: &Path) -> Label {
    let label = Label::new(None);
    label.add_css_class("obsidian-status-label");
    label.add_css_class("obsidian-status-ok");
    update_status_label(&label, read_status_code(status_path));
    watch_status_file(&label, status_path);
    label
}

fn watch_status_file(label: &Label, status_path: &Path) {
    let label = label.clone();
    let status_path = status_path.to_path_buf();
    let last_status = Cell::new(read_status_code(&status_path));

    glib::timeout_add_local(std::time::Duration::from_millis(180), move || {
        let next_status = read_status_code(&status_path);
        if next_status != last_status.get() {
            update_status_label(&label, next_status);
            last_status.set(next_status);
        }

        glib::ControlFlow::Continue
    });
}

fn read_status_code(status_path: &Path) -> i32 {
    fs::read_to_string(status_path)
        .ok()
        .and_then(|text| text.trim().parse::<i32>().ok())
        .unwrap_or(0)
}

fn update_status_label(label: &Label, status_code: i32) {
    let text = if status_code == 0 {
        "ready".to_string()
    } else {
        format!("failed {status_code}")
    };
    label.set_text(&text);
    label.remove_css_class("obsidian-status-ok");
    label.remove_css_class("obsidian-status-error");
    if status_code == 0 {
        label.add_css_class("obsidian-status-ok");
    } else {
        label.add_css_class("obsidian-status-error");
    }
}
