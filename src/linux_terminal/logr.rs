use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc::{Receiver, TryRecvError},
    time::Duration,
};

use gtk::{
    glib, prelude::*, Box as GtkBox, Button, Entry, Label, ListBox, ListBoxRow, MenuButton,
    Orientation, Popover, ScrolledWindow,
};

use crate::{
    features::logs::{load_source, spawn_file_follower, LogEntry},
    logger,
};

const LOG_EXTENSIONS: &[&str] = &["log", "jsonl", "json", "txt"];
const MAX_SCAN_DEPTH: usize = 3;
const MAX_VISIBLE_ENTRIES: usize = 200;

pub(super) fn build_logr_pane() -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.add_css_class("obsidian-logr-root");

    // --- Header: title + entry count ---
    let header = GtkBox::new(Orientation::Horizontal, 6);
    header.add_css_class("obsidian-logr-header");

    let title = Label::new(Some("logr"));
    title.add_css_class("obsidian-logr-title");
    title.set_xalign(0.0);
    title.set_hexpand(true);

    let count_label = Label::new(Some("0"));
    count_label.add_css_class("obsidian-logr-count");

    header.append(&title);
    header.append(&count_label);
    root.append(&header);

    // --- File picker: custom dropdown ---
    let picker_row = GtkBox::new(Orientation::Horizontal, 4);
    picker_row.add_css_class("obsidian-logr-picker");

    let popover_list = ListBox::new();
    popover_list.set_selection_mode(gtk::SelectionMode::Single);
    popover_list.add_css_class("obsidian-logr-popover-list");

    let popover_scroller = ScrolledWindow::new();
    popover_scroller.set_child(Some(&popover_list));
    popover_scroller.set_min_content_height(120);
    popover_scroller.set_max_content_height(280);
    popover_scroller.set_propagate_natural_height(true);
    popover_scroller.set_min_content_width(200);

    let popover = Popover::new();
    popover.set_child(Some(&popover_scroller));
    popover.set_has_arrow(false);
    popover.add_css_class("obsidian-logr-popover");

    let select_button = MenuButton::builder()
        .label("select log file...")
        .css_classes(["obsidian-logr-select"])
        .hexpand(true)
        .popover(&popover)
        .build();

    let refresh_button = Button::builder()
        .icon_name("view-refresh-symbolic")
        .css_classes(["obsidian-logr-icon-btn"])
        .tooltip_text("Rescan for log files")
        .build();

    picker_row.append(&select_button);
    picker_row.append(&refresh_button);
    root.append(&picker_row);

    // --- Controls: play/stop ---
    let controls_row = GtkBox::new(Orientation::Horizontal, 4);
    controls_row.add_css_class("obsidian-logr-controls");

    let play_button = Button::builder()
        .icon_name("media-playback-start-symbolic")
        .css_classes(["obsidian-logr-icon-btn"])
        .tooltip_text("Start live stream")
        .sensitive(false)
        .build();

    let stop_button = Button::builder()
        .icon_name("media-playback-stop-symbolic")
        .css_classes(["obsidian-logr-icon-btn"])
        .tooltip_text("Stop live stream")
        .sensitive(false)
        .visible(false)
        .build();

    let stream_label = Label::new(Some("select a file to stream"));
    stream_label.add_css_class("obsidian-logr-stream-label");
    stream_label.set_xalign(0.0);
    stream_label.set_hexpand(true);

    controls_row.append(&play_button);
    controls_row.append(&stop_button);
    controls_row.append(&stream_label);
    root.append(&controls_row);

    // --- Filter entry ---
    let filter_entry = Entry::new();
    filter_entry.add_css_class("obsidian-logr-filter");
    filter_entry.set_placeholder_text(Some("filter..."));
    filter_entry.set_hexpand(true);
    root.append(&filter_entry);

    // --- Log list ---
    let list = ListBox::new();
    list.set_selection_mode(gtk::SelectionMode::None);
    list.add_css_class("obsidian-log-list");

    let scroller = ScrolledWindow::new();
    scroller.set_hexpand(true);
    scroller.set_vexpand(true);
    scroller.set_child(Some(&list));
    root.append(&scroller);

    // --- Status bar ---
    let status = Label::new(Some("idle"));
    status.add_css_class("obsidian-logr-status");
    status.set_xalign(0.0);
    status.set_ellipsize(gtk::pango::EllipsizeMode::End);
    root.append(&status);

    // --- State ---
    let state = Rc::new(RefCell::new(PaneState::default()));
    let discovered = Rc::new(RefCell::new(Vec::<PathBuf>::new()));
    let selected_path: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));

    // Initial scan
    populate_popover_list(&popover_list, &discovered);

    // Refresh button: rescan
    {
        let popover_list = popover_list.clone();
        let discovered = discovered.clone();
        refresh_button.connect_clicked(move |_| {
            populate_popover_list(&popover_list, &discovered);
        });
    }

    // Popover row activated: just select the file (don't load yet)
    {
        let discovered = discovered.clone();
        let select_button = select_button.clone();
        let selected_path = selected_path.clone();
        let play_button = play_button.clone();
        let stream_label = stream_label.clone();
        let popover = popover.clone();
        popover_list.connect_row_activated(move |_, row| {
            let idx = row.index() as usize;
            let files = discovered.borrow();
            if let Some(path) = files.get(idx) {
                let label = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                select_button.set_label(label);
                stream_label.set_text(&format!("ready: {label}"));
                *selected_path.borrow_mut() = Some(path.clone());
                play_button.set_sensitive(true);
            }
            popover.popdown();
        });
    }

    // Play button: load + start following
    {
        let list = list.clone();
        let count_label = count_label.clone();
        let status = status.clone();
        let state = state.clone();
        let filter_entry = filter_entry.clone();
        let selected_path = selected_path.clone();
        let play_btn = play_button.clone();
        let stop_button = stop_button.clone();
        let stream_label = stream_label.clone();
        play_button.connect_clicked(move |_| {
            let play_button = &play_btn;
            let path = selected_path.borrow().clone();
            if let Some(path) = path {
                load_file(
                    &path,
                    &list,
                    &count_label,
                    &status,
                    &state,
                    &filter_entry,
                );
                play_button.set_visible(false);
                play_button.set_sensitive(false);
                stop_button.set_visible(true);
                stop_button.set_sensitive(true);
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                stream_label.set_text(&format!("streaming: {name}"));
            }
        });
    }

    // Stop button: stop following, keep entries visible
    {
        let state = state.clone();
        let play_button_ref = play_button.clone();
        let stop_button_ref = stop_button.clone();
        let stream_label = stream_label.clone();
        let status = status.clone();
        stop_button.connect_clicked(move |_| {
            {
                let mut s = state.borrow_mut();
                s.follower = None;
                s.last_status = "stopped".to_string();
            }
            stop_button_ref.set_visible(false);
            stop_button_ref.set_sensitive(false);
            play_button_ref.set_visible(true);
            play_button_ref.set_sensitive(true);
            stream_label.set_text("stopped");
            status.set_text("stopped");
        });
    }

    // Filter changed
    {
        let list = list.clone();
        let count_label = count_label.clone();
        let status = status.clone();
        let state = state.clone();
        filter_entry.connect_changed(move |entry| {
            state.borrow_mut().query = entry.text().to_string();
            refresh_view(&list, &count_label, &status, &state.borrow());
        });
    }

    // Live follower poll
    watch_follower(&list, &count_label, &status, &state);

    root
}

#[derive(Default)]
struct PaneState {
    entries: Vec<LogEntry>,
    follower: Option<Receiver<LogEntry>>,
    query: String,
    last_status: String,
}

// --- File discovery ---

fn populate_popover_list(popover_list: &ListBox, discovered: &Rc<RefCell<Vec<PathBuf>>>) {
    while let Some(child) = popover_list.first_child() {
        popover_list.remove(&child);
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut files = Vec::new();
    walk_for_logs(&cwd, 0, &mut files);
    files.sort();

    for file in &files {
        let display = file
            .strip_prefix(&cwd)
            .unwrap_or(file)
            .display()
            .to_string();

        let label = Label::new(Some(&display));
        label.set_xalign(0.0);
        label.add_css_class("obsidian-logr-popover-item");

        let row = ListBoxRow::new();
        row.set_child(Some(&label));
        row.add_css_class("obsidian-logr-popover-row");
        popover_list.append(&row);
    }

    *discovered.borrow_mut() = files;
}

fn walk_for_logs(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_SCAN_DEPTH {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
        }

        if path.is_dir() {
            walk_for_logs(&path, depth + 1, out);
        } else if is_log_file(&path) {
            out.push(path);
        }
    }
}

fn is_log_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| LOG_EXTENSIONS.contains(&ext))
}

// --- Load ---

fn load_file(
    path: &Path,
    list: &ListBox,
    count_label: &Label,
    status: &Label,
    state: &Rc<RefCell<PaneState>>,
    filter_entry: &Entry,
) {
    let path_str = path.display().to_string();
    logger::info("logr: loading file", &[("path", &path_str)]);
    match load_source(Some(path_str.clone())) {
        Ok(source) => {
            let follower = source.follow_config.map(spawn_file_follower);
            let loaded = source.entries.len();
            let mode = if follower.is_some() { "live" } else { "static" };
            logger::info("logr: file loaded", &[
                ("entries", &loaded.to_string()),
                ("mode", mode),
            ]);
            let query = filter_entry.text().to_string();
            *state.borrow_mut() = PaneState {
                entries: source.entries,
                follower,
                query,
                last_status: format!("{loaded} entries ({mode})"),
            };
            refresh_view(list, count_label, status, &state.borrow());
        }
        Err(error) => {
            logger::error("logr: file load failed", &[("error", &error.to_string())]);
            state.borrow_mut().last_status = format!("error: {error}");
            refresh_view(list, count_label, status, &state.borrow());
        }
    }
}

// --- Live follower ---

fn watch_follower(
    list: &ListBox,
    count_label: &Label,
    status: &Label,
    state: &Rc<RefCell<PaneState>>,
) {
    let list = list.clone();
    let count_label = count_label.clone();
    let status = status.clone();
    let state = state.clone();
    glib::timeout_add_local(Duration::from_millis(250), move || {
        if drain_followed_entries(&state) {
            refresh_view(&list, &count_label, &status, &state.borrow());
        }
        glib::ControlFlow::Continue
    });
}

fn drain_followed_entries(state: &Rc<RefCell<PaneState>>) -> bool {
    let mut new_entries = Vec::new();
    let mut stopped = false;
    {
        let mut s = state.borrow_mut();
        if let Some(receiver) = s.follower.as_mut() {
            loop {
                match receiver.try_recv() {
                    Ok(entry) => new_entries.push(entry),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        stopped = true;
                        break;
                    }
                }
            }
        }
    }

    if !new_entries.is_empty() || stopped {
        let mut s = state.borrow_mut();
        s.entries.append(&mut new_entries);
        if stopped {
            s.follower = None;
            s.last_status = "follow ended".to_string();
        }
        true
    } else {
        false
    }
}

// --- View rendering ---

fn refresh_view(list: &ListBox, count_label: &Label, status: &Label, state: &PaneState) {
    clear_list(list);

    let filtered: Vec<&LogEntry> = state
        .entries
        .iter()
        .filter(|e| e.matches_query(&state.query))
        .collect();
    let shown = filtered.len();
    let total = state.entries.len();
    let start = shown.saturating_sub(MAX_VISIBLE_ENTRIES);

    count_label.set_text(&format!("{shown}/{total}"));

    status.set_text(if state.last_status.is_empty() {
        "idle"
    } else {
        &state.last_status
    });

    if shown == 0 {
        let msg = if total == 0 {
            "no logs loaded"
        } else {
            "no matches"
        };
        let lbl = Label::new(Some(msg));
        lbl.add_css_class("obsidian-logr-empty");
        lbl.set_xalign(0.0);
        list.append(&lbl);
        return;
    }

    for entry in filtered.into_iter().skip(start) {
        list.append(&entry_row(entry));
    }
}

fn entry_row(entry: &LogEntry) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("obsidian-log-entry");

    let level = entry.level_label().to_lowercase();
    row.add_css_class(&format!("log-{level}"));

    let badge = Label::new(Some(&level.chars().next().unwrap_or('?').to_uppercase().to_string()));
    badge.add_css_class("log-level-dot");

    let body = Label::new(Some(entry.raw_line()));
    body.set_xalign(0.0);
    body.set_ellipsize(gtk::pango::EllipsizeMode::End);
    body.set_hexpand(true);
    body.add_css_class("log-body");

    row.append(&badge);
    row.append(&body);
    row
}

fn clear_list(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}
