mod history;
mod prompt;
mod status;

use std::{cell::Cell, path::Path, rc::Rc};

use gtk::{gdk, prelude::*, Box as GtkBox, Button, Entry, EventControllerKey, Orientation, Separator};
use vte4::{prelude::*, Regex, Terminal};

use self::{
    history::{record_history, InputHistory},
    prompt::build_prompt_box,
    status::build_status_label,
};

pub(super) fn append_input_row(
    container: &GtkBox,
    terminal: &Terminal,
    status_path: &Path,
) -> Entry {
    let separator = Separator::new(Orientation::Horizontal);
    separator.add_css_class("obsidian-separator");
    container.append(&separator);

    let input_container = GtkBox::new(Orientation::Horizontal, 8);
    input_container.add_css_class("obsidian-input-pill");

    let prompt_container = build_prompt_box(terminal);
    input_container.append(&prompt_container);

    let status_label = build_status_label(status_path);
    input_container.append(&status_label);

    let entry = Entry::new();
    entry.add_css_class("obsidian-entry");
    entry.set_hexpand(true);
    entry.set_placeholder_text(Some("Enter command"));
    input_container.append(&entry);

    // Search nav (only visible in search mode)
    let prev_button = tool_button("go-up-symbolic");
    let next_button = tool_button("go-down-symbolic");
    prev_button.set_visible(false);
    next_button.set_visible(false);
    input_container.append(&prev_button);
    input_container.append(&next_button);

    // Search toggle — small, right end
    let search_button = Button::builder()
        .icon_name("system-search-symbolic")
        .css_classes(["obsidian-search-toggle"])
        .build();
    input_container.append(&search_button);

    container.append(&input_container);

    // --- Wire everything ---

    let search_mode = Rc::new(Cell::new(false));
    let history = Rc::new(InputHistory::default());

    // Activate (Enter): run command or search next depending on mode
    wire_activate(
        &entry,
        terminal,
        &history,
        &search_mode,
    );

    // Key handler: Ctrl+F toggle, Escape exit search, clipboard, history
    wire_keys(
        &entry,
        terminal,
        &history,
        &search_mode,
        &prompt_container,
        &status_label,
        &prev_button,
        &next_button,
    );

    // Entry text changed: apply search regex in search mode
    wire_search_on_change(&entry, terminal, &search_mode);

    // Search button: toggle search mode
    wire_search_toggle(
        &search_button,
        &entry,
        terminal,
        &search_mode,
        &prompt_container,
        &status_label,
        &prev_button,
        &next_button,
    );

    // Search prev/next
    wire_search_nav(terminal, &prev_button, &next_button);

    entry
}

fn tool_button(icon_name: &str) -> Button {
    Button::builder()
        .icon_name(icon_name)
        .css_classes(["obsidian-tool-button"])
        .build()
}

// --- Activate (Enter key) ---

fn wire_activate(
    entry: &Entry,
    terminal: &Terminal,
    history: &Rc<InputHistory>,
    search_mode: &Rc<Cell<bool>>,
) {
    let terminal = terminal.clone();
    let history = history.clone();
    let search_mode = search_mode.clone();

    entry.connect_activate(move |entry| {
        let text = entry.text();
        if text.is_empty() {
            return;
        }

        if search_mode.get() {
            // Search mode: Enter = find next
            let _ = terminal.search_find_next();
        } else {
            // Command mode: Enter = execute
            record_history(&history, text.as_str());
            let mut input = text.to_string();
            input.push('\n');
            terminal.feed_child(input.as_bytes());
            entry.set_text("");
            let _ = entry.grab_focus_without_selecting();
        }
    });
}

// --- Keyboard handler ---

fn wire_keys(
    entry: &Entry,
    terminal: &Terminal,
    history: &Rc<InputHistory>,
    search_mode: &Rc<Cell<bool>>,
    prompt: &GtkBox,
    status: &gtk::Label,
    prev_btn: &Button,
    next_btn: &Button,
) {
    let controller = EventControllerKey::new();

    let entry_ref = entry.clone();
    let terminal_ref = terminal.clone();
    let history_ref = history.clone();
    let search_mode_ref = search_mode.clone();
    let prompt_ref = prompt.clone();
    let status_ref = status.clone();
    let prev_ref = prev_btn.clone();
    let next_ref = next_btn.clone();

    controller.connect_key_pressed(move |_, key, _, modifiers| {
        let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
        let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);
        let alt = modifiers.contains(gdk::ModifierType::ALT_MASK);

        // Ctrl+F: toggle search mode
        if ctrl && !shift && !alt && matches!(key.to_unicode(), Some('f') | Some('F')) {
            let entering = !search_mode_ref.get();
            set_search_mode(
                entering,
                &search_mode_ref,
                &entry_ref,
                &terminal_ref,
                &prompt_ref,
                &status_ref,
                &prev_ref,
                &next_ref,
            );
            return gtk::glib::Propagation::Stop;
        }

        // Escape: exit search mode
        if key == gdk::Key::Escape && search_mode_ref.get() {
            set_search_mode(
                false,
                &search_mode_ref,
                &entry_ref,
                &terminal_ref,
                &prompt_ref,
                &status_ref,
                &prev_ref,
                &next_ref,
            );
            return gtk::glib::Propagation::Stop;
        }

        // Terminal clipboard/control shortcuts
        if history::handle_clipboard_shortcuts(&entry_ref, &terminal_ref, key, modifiers) {
            return gtk::glib::Propagation::Stop;
        }

        // History navigation (only in command mode)
        if !search_mode_ref.get()
            && history::handle_history_navigation(&entry_ref, &history_ref, key)
        {
            return gtk::glib::Propagation::Stop;
        }

        gtk::glib::Propagation::Proceed
    });

    entry.add_controller(controller);
}

// --- Search mode toggle ---

fn set_search_mode(
    enabled: bool,
    search_mode: &Rc<Cell<bool>>,
    entry: &Entry,
    terminal: &Terminal,
    prompt: &GtkBox,
    status: &gtk::Label,
    prev_btn: &Button,
    next_btn: &Button,
) {
    search_mode.set(enabled);

    // Toggle class on the parent pill container so CSS can style it
    if let Some(pill) = entry.parent().and_then(|p| p.downcast::<GtkBox>().ok()) {
        if enabled {
            pill.add_css_class("search-active");
        } else {
            pill.remove_css_class("search-active");
        }
    }

    if enabled {
        entry.set_placeholder_text(Some("Search output..."));
        entry.add_css_class("search-active");
        prompt.set_visible(false);
        status.set_visible(false);
        prev_btn.set_visible(true);
        next_btn.set_visible(true);
        // Apply current text as search immediately
        apply_search(terminal, entry.text().as_str());
    } else {
        entry.set_placeholder_text(Some("Enter command"));
        entry.remove_css_class("search-active");
        prompt.set_visible(true);
        status.set_visible(true);
        prev_btn.set_visible(false);
        next_btn.set_visible(false);
        // Clear search highlight
        terminal.search_set_regex(None, 0);
    }

    entry.set_text("");
    let _ = entry.grab_focus_without_selecting();
}

fn wire_search_toggle(
    search_button: &Button,
    entry: &Entry,
    terminal: &Terminal,
    search_mode: &Rc<Cell<bool>>,
    prompt: &GtkBox,
    status: &gtk::Label,
    prev_btn: &Button,
    next_btn: &Button,
) {
    let entry = entry.clone();
    let terminal = terminal.clone();
    let search_mode = search_mode.clone();
    let prompt = prompt.clone();
    let status = status.clone();
    let prev_btn = prev_btn.clone();
    let next_btn = next_btn.clone();

    search_button.connect_clicked(move |_| {
        let entering = !search_mode.get();
        set_search_mode(
            entering,
            &search_mode,
            &entry,
            &terminal,
            &prompt,
            &status,
            &prev_btn,
            &next_btn,
        );
    });
}

// --- Search: live filtering as user types ---

fn wire_search_on_change(entry: &Entry, terminal: &Terminal, search_mode: &Rc<Cell<bool>>) {
    let terminal = terminal.clone();
    let search_mode = search_mode.clone();

    entry.connect_changed(move |entry| {
        if search_mode.get() {
            apply_search(&terminal, entry.text().as_str());
        }
    });
}

fn apply_search(terminal: &Terminal, query: &str) {
    if query.is_empty() {
        terminal.search_set_regex(None, 0);
        return;
    }

    match Regex::for_search(query, 0) {
        Ok(regex) => {
            terminal.search_set_wrap_around(true);
            terminal.search_set_regex(Some(&regex), 0);
            let _ = terminal.search_find_next();
        }
        Err(error) => {
            eprintln!("search regex invalid: {error}");
        }
    }
}

fn wire_search_nav(terminal: &Terminal, prev_button: &Button, next_button: &Button) {
    let t = terminal.clone();
    prev_button.connect_clicked(move |_| {
        let _ = t.search_find_previous();
    });

    let t = terminal.clone();
    next_button.connect_clicked(move |_| {
        let _ = t.search_find_next();
    });
}

