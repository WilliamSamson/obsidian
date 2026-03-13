use std::{
    cell::RefCell,
    env,
    fs,
    path::PathBuf,
    rc::Rc,
};

use gtk::{
    prelude::*, Align, Box as GtkBox, Button, DropDown, Entry, Label, Orientation, ScrolledWindow,
    SpinButton, StringList, Switch,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Settings {
    pub(super) font_family: String,
    pub(super) font_size: u32,
    pub(super) scrollback_lines: u32,
    pub(super) cursor_style: String,
    pub(super) cursor_blink: bool,
    pub(super) shell: String,
    pub(super) logr_panel_open: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        Self {
            font_family: "DejaVu Sans Mono".to_string(),
            font_size: 10,
            scrollback_lines: 20_000,
            cursor_style: "ibeam".to_string(),
            cursor_blink: false,
            shell,
            logr_panel_open: true,
        }
    }
}

pub(super) fn load_settings() -> Settings {
    let path = settings_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub(super) fn save_settings(settings: &Settings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, json);
    }
}

fn settings_path() -> PathBuf {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("obsidian").join("settings.json")
}

// --- UI ---

pub(super) fn build_settings_page(on_back: impl Fn() + 'static) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.set_hexpand(true);
    root.add_css_class("obsidian-settings-root");

    // Header
    let header = GtkBox::new(Orientation::Horizontal, 8);
    header.add_css_class("obsidian-settings-header");

    let back_button = Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(["obsidian-settings-back"])
        .tooltip_text("Back")
        .build();
    back_button.connect_clicked(move |_| on_back());

    let title = Label::new(Some("settings"));
    title.add_css_class("obsidian-settings-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);

    header.append(&back_button);
    header.append(&title);
    root.append(&header);

    // Scrollable content
    let content = GtkBox::new(Orientation::Vertical, 0);
    content.add_css_class("obsidian-settings-content");

    let scroller = ScrolledWindow::new();
    scroller.set_vexpand(true);
    scroller.set_hexpand(true);
    scroller.set_child(Some(&content));
    root.append(&scroller);

    let settings = Rc::new(RefCell::new(load_settings()));

    // --- Terminal section ---
    content.append(&section_label("terminal"));

    // Font family
    {
        let s = settings.clone();
        let entry = text_row(&content, "font family", &settings.borrow().font_family);
        entry.connect_changed(move |e| {
            s.borrow_mut().font_family = e.text().to_string();
        });
    }

    // Font size
    {
        let s = settings.clone();
        let spin = spin_row(&content, "font size", settings.borrow().font_size as f64, 6.0, 32.0);
        spin.connect_value_changed(move |sp| {
            s.borrow_mut().font_size = sp.value() as u32;
        });
    }

    // Scrollback lines
    {
        let s = settings.clone();
        let spin = spin_row(
            &content,
            "scrollback lines",
            settings.borrow().scrollback_lines as f64,
            500.0,
            100_000.0,
        );
        spin.set_digits(0);
        spin.set_increments(500.0, 5000.0);
        spin.connect_value_changed(move |sp| {
            s.borrow_mut().scrollback_lines = sp.value() as u32;
        });
    }

    // Cursor style
    {
        let s = settings.clone();
        let styles = &["ibeam", "block", "underline"];
        let current = &settings.borrow().cursor_style;
        let active = styles.iter().position(|s| s == current).unwrap_or(0);
        let dropdown = dropdown_row(&content, "cursor style", styles, active as u32);
        dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            let val = match idx {
                1 => "block",
                2 => "underline",
                _ => "ibeam",
            };
            s.borrow_mut().cursor_style = val.to_string();
        });
    }

    // Cursor blink
    {
        let s = settings.clone();
        let switch = switch_row(&content, "cursor blink", settings.borrow().cursor_blink);
        switch.connect_state_set(move |_, active| {
            s.borrow_mut().cursor_blink = active;
            gtk::glib::Propagation::Proceed
        });
    }

    // --- Shell section ---
    content.append(&section_label("shell"));

    // Shell path
    {
        let s = settings.clone();
        let entry = text_row(&content, "shell", &settings.borrow().shell);
        entry.connect_changed(move |e| {
            s.borrow_mut().shell = e.text().to_string();
        });
    }

    // --- Logr section ---
    content.append(&section_label("logr"));

    // Panel open on start
    {
        let s = settings.clone();
        let switch = switch_row(&content, "panel open on start", settings.borrow().logr_panel_open);
        switch.connect_state_set(move |_, active| {
            s.borrow_mut().logr_panel_open = active;
            gtk::glib::Propagation::Proceed
        });
    }

    // --- About section ---
    content.append(&section_label("about"));
    content.append(&info_row("version", env!("CARGO_PKG_VERSION")));
    content.append(&info_row("config", &settings_path().display().to_string()));

    // --- Save button ---
    let footer = GtkBox::new(Orientation::Horizontal, 0);
    footer.add_css_class("obsidian-settings-footer");
    footer.set_halign(Align::End);

    let save_button = Button::builder()
        .label("save")
        .css_classes(["obsidian-settings-save"])
        .build();
    {
        let settings = settings.clone();
        save_button.connect_clicked(move |btn| {
            save_settings(&settings.borrow());
            btn.set_label("saved");
            let btn = btn.clone();
            gtk::glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
                btn.set_label("save");
            });
        });
    }
    footer.append(&save_button);
    content.append(&footer);

    root
}

// --- Row builders ---

fn section_label(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("obsidian-settings-section");
    label.set_xalign(0.0);
    label
}

fn setting_row(parent: &GtkBox, label_text: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("obsidian-settings-row");

    let label = Label::new(Some(label_text));
    label.add_css_class("obsidian-settings-label");
    label.set_xalign(0.0);
    label.set_hexpand(true);

    row.append(&label);
    parent.append(&row);
    row
}

fn text_row(parent: &GtkBox, label_text: &str, value: &str) -> Entry {
    let row = setting_row(parent, label_text);
    let entry = Entry::new();
    entry.set_text(value);
    entry.add_css_class("obsidian-settings-entry");
    entry.set_width_chars(20);
    entry.set_halign(Align::End);
    row.append(&entry);
    entry
}

fn spin_row(parent: &GtkBox, label_text: &str, value: f64, min: f64, max: f64) -> SpinButton {
    let row = setting_row(parent, label_text);
    let spin = SpinButton::with_range(min, max, 1.0);
    spin.set_value(value);
    spin.add_css_class("obsidian-settings-spin");
    spin.set_halign(Align::End);
    row.append(&spin);
    spin
}

fn switch_row(parent: &GtkBox, label_text: &str, active: bool) -> Switch {
    let row = setting_row(parent, label_text);
    let switch = Switch::new();
    switch.set_active(active);
    switch.add_css_class("obsidian-settings-switch");
    switch.set_halign(Align::End);
    switch.set_valign(Align::Center);
    row.append(&switch);
    switch
}

fn dropdown_row(parent: &GtkBox, label_text: &str, items: &[&str], active: u32) -> DropDown {
    let row = setting_row(parent, label_text);
    let model = StringList::new(items);
    let dropdown = DropDown::builder().model(&model).selected(active).build();
    dropdown.add_css_class("obsidian-settings-dropdown");
    dropdown.set_halign(Align::End);
    row.append(&dropdown);
    dropdown
}

fn info_row(label_text: &str, value: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("obsidian-settings-row");

    let label = Label::new(Some(label_text));
    label.add_css_class("obsidian-settings-label");
    label.set_xalign(0.0);
    label.set_hexpand(true);

    let val = Label::new(Some(value));
    val.add_css_class("obsidian-settings-value");
    val.set_xalign(1.0);
    val.set_selectable(true);

    row.append(&label);
    row.append(&val);
    row
}
