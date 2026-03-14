mod ops;
mod tab_strip;

use std::{cell::RefCell, rc::Rc};

use gtk::{
    gdk, prelude::*, Align, Box as GtkBox, Button, EventControllerKey, Notebook, Orientation,
    PackType, PolicyType, ScrolledWindow,
};

use super::{
    persist::{self, TabSnapshot, WorkspaceSnapshot},
    profile::{next_profile, ProfileId},
    settings::Settings,
    tab::TabView,
};

pub(super) struct WorkspaceView {
    root: GtkBox,
    notebook: Notebook,
    tab_container: GtkBox,
    tab_scroller: ScrolledWindow,
    tabs: Rc<RefCell<Vec<TabView>>>,
    settings: Rc<RefCell<Settings>>,
}

impl WorkspaceView {
    pub(super) fn new(settings: Rc<RefCell<Settings>>) -> Self {
        let root = GtkBox::new(Orientation::Vertical, 0);
        root.set_vexpand(true);
        root.set_focusable(true);

        let tab_container = GtkBox::new(Orientation::Horizontal, 4);
        tab_container.add_css_class("obsidian-tabs-list");
        tab_container.set_valign(Align::Center);

        let add_button = action_button("list-add-symbolic");
        add_button.add_css_class("obsidian-add-tab-button");
        add_button.set_halign(Align::End);
        add_button.set_valign(Align::Center);

        let (tab_bar_row, tab_scroller) = tab_bar_row(&tab_container, &add_button);
        let notebook = notebook();
        let (split_tab, close_tab, profile_tab, actions_box) = actions_box();
        notebook.set_action_widget(&actions_box, PackType::End);

        root.append(&tab_bar_row);
        root.append(&notebook);

        let tabs = Rc::new(RefCell::new(Vec::new()));
        let workspace = Self {
            root,
            notebook,
            tab_container,
            tab_scroller,
            tabs,
            settings,
        };

        workspace.restore();
        workspace.bind_actions(add_button, split_tab, close_tab, profile_tab);
        workspace.bind_keyboard_shortcuts();
        workspace.rebuild_tab_strip();
        workspace
    }

    pub(super) fn root(&self) -> &GtkBox {
        &self.root
    }

    pub(super) fn apply_settings(&self, settings: &Settings) {
        *self.settings.borrow_mut() = settings.clone();
        for tab in self.tabs.borrow().iter() {
            tab.apply_settings(settings);
        }
    }

    pub(super) fn save(&self) {
        let snapshot = WorkspaceSnapshot {
            active_tab: current_index(&self.notebook),
            tabs: self.tabs.borrow().iter().map(TabView::to_snapshot).collect(),
        };
        if let Err(error) = persist::save_workspace(&snapshot) {
            eprintln!("workspace save failed: {error}");
        }
    }

    fn restore(&self) {
        let snapshot = persist::load_workspace().ok().flatten().unwrap_or_else(default_snapshot);
        for tab in snapshot.tabs {
            self.append_tab(tab);
        }
        self.notebook.set_current_page(Some(snapshot.active_tab as u32));
    }

    fn bind_actions(&self, new_tab: Button, split_tab: Button, close_tab: Button, profile_tab: Button) {
        let notebook = self.notebook.clone();
        let tabs = self.tabs.clone();
        let tab_container = self.tab_container.clone();
        let tab_scroller = self.tab_scroller.clone();
        let settings = self.settings.clone();
        new_tab.connect_clicked(move |_| {
            create_new_tab(&tabs, &notebook, &tab_container, &tab_scroller, &settings);
        });

        let notebook = self.notebook.clone();
        let tabs = self.tabs.clone();
        split_tab.connect_clicked(move |_| {
            if let Some(tab) = tabs.borrow_mut().get_mut(current_index(&notebook)) {
                tab.toggle_split();
            }
        });

        let notebook = self.notebook.clone();
        let tabs = self.tabs.clone();
        let tab_container = self.tab_container.clone();
        close_tab.connect_clicked(move |_| {
            let _ = ops::close_tab_at(&tabs, &notebook, current_index(&notebook));
            tab_strip::rebuild_tab_strip(&tab_container, &notebook, &tabs);
        });

        let notebook = self.notebook.clone();
        let tabs = self.tabs.clone();
        let tab_container = self.tab_container.clone();
        profile_tab.connect_clicked(move |_| {
            if let Some(tab) = tabs.borrow_mut().get_mut(current_index(&notebook)) {
                let next = next_profile(tab.profile_id());
                tab.cycle_profile(next);
            }
            tab_strip::rebuild_tab_strip(&tab_container, &notebook, &tabs);
        });

        // Rebuild on tab switch so the active indicator and control state always match the page.
        let tabs = self.tabs.clone();
        let tab_container = self.tab_container.clone();
        let tab_scroller = self.tab_scroller.clone();
        self.notebook.connect_switch_page(move |notebook, _, page_num| {
            let active = page_num as usize;
            tab_strip::rebuild_tab_strip_at(&tab_container, notebook, &tabs, active);
            tab_strip::reveal_active_tab_at(&tab_container, &tab_scroller, active);
        });

        // On tab removal: full rebuild needed since widget count changed
        let tabs = self.tabs.clone();
        let tab_container = self.tab_container.clone();
        let tab_scroller = self.tab_scroller.clone();
        self.notebook.connect_page_removed(move |notebook, _, _| {
            tab_strip::rebuild_tab_strip(&tab_container, notebook, &tabs);
            tab_strip::update_active_tab(&tab_container, notebook);
            tab_strip::reveal_active_tab(&tab_container, &tab_scroller, notebook);
        });
    }

    fn bind_keyboard_shortcuts(&self) {
        let controller = EventControllerKey::new();
        controller.set_propagation_phase(gtk::PropagationPhase::Capture);

        let notebook = self.notebook.clone();
        let tabs = self.tabs.clone();
        let tab_container = self.tab_container.clone();
        let tab_scroller = self.tab_scroller.clone();
        let settings = self.settings.clone();

        controller.connect_key_pressed(move |_, key, _, modifiers| {
            let ctrl = modifiers.contains(gdk::ModifierType::CONTROL_MASK);
            let shift = modifiers.contains(gdk::ModifierType::SHIFT_MASK);

            if !ctrl {
                return gtk::glib::Propagation::Proceed;
            }

            match key {
                // Ctrl+T: New tab
                gdk::Key::t if !shift => {
                    create_new_tab(&tabs, &notebook, &tab_container, &tab_scroller, &settings);
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+W: Close current tab
                gdk::Key::w if !shift => {
                    let _ = ops::close_tab_at(&tabs, &notebook, current_index(&notebook));
                    tab_strip::rebuild_tab_strip(&tab_container, &notebook, &tabs);
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Tab: Next tab
                gdk::Key::Tab if !shift => {
                    let count = notebook.n_pages() as usize;
                    if count > 1 {
                        let next = (current_index(&notebook) + 1) % count;
                        notebook.set_current_page(Some(next as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Shift+Tab: Previous tab
                gdk::Key::Tab if shift => {
                    let count = notebook.n_pages() as usize;
                    if count > 1 {
                        let current = current_index(&notebook);
                        let prev = if current == 0 { count - 1 } else { current - 1 };
                        notebook.set_current_page(Some(prev as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Shift+Left: Move tab left
                gdk::Key::Left if shift => {
                    let count = notebook.n_pages() as usize;
                    let current = current_index(&notebook);
                    if count > 1 && current > 0 {
                        ops::reorder_tab(&tabs, &notebook, current, current - 1);
                        tab_strip::rebuild_tab_strip(&tab_container, &notebook, &tabs);
                        notebook.set_current_page(Some((current - 1) as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+Shift+Right: Move tab right
                gdk::Key::Right if shift => {
                    let count = notebook.n_pages() as usize;
                    let current = current_index(&notebook);
                    if count > 1 && current + 1 < count {
                        ops::reorder_tab(&tabs, &notebook, current, current + 1);
                        tab_strip::rebuild_tab_strip(&tab_container, &notebook, &tabs);
                        notebook.set_current_page(Some((current + 1) as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                // Ctrl+1-9: Jump to tab by number
                _ if key.to_unicode().is_some_and(|c| ('1'..='9').contains(&c)) => {
                    let target = (key.to_unicode().unwrap() as usize) - ('1' as usize);
                    let count = notebook.n_pages() as usize;
                    if target < count {
                        notebook.set_current_page(Some(target as u32));
                    }
                    gtk::glib::Propagation::Stop
                }
                _ => gtk::glib::Propagation::Proceed,
            }
        });

        self.root.add_controller(controller);
    }

    fn append_tab(&self, snapshot: TabSnapshot) {
        append_tab(&self.tabs, &self.notebook, snapshot, &self.settings);
    }

    fn rebuild_tab_strip(&self) {
        tab_strip::rebuild_tab_strip(&self.tab_container, &self.notebook, &self.tabs);
    }
}

fn create_new_tab(
    tabs: &Rc<RefCell<Vec<TabView>>>,
    notebook: &Notebook,
    tab_container: &GtkBox,
    tab_scroller: &ScrolledWindow,
    settings: &Rc<RefCell<Settings>>,
) {
    let next_index = tabs.borrow().len() + 1;
    let snapshot = TabSnapshot {
        title: format!("tab {next_index}"),
        profile: ProfileId::Default,
        left_cwd: None,
        right_cwd: None,
    };
    append_tab(tabs, notebook, snapshot, settings);
    notebook.set_current_page(Some((tabs.borrow().len().saturating_sub(1)) as u32));
    tab_strip::rebuild_tab_strip(tab_container, notebook, tabs);
    tab_strip::update_active_tab(tab_container, notebook);
    tab_strip::reveal_active_tab(tab_container, tab_scroller, notebook);
}

fn tab_bar_row(tab_container: &GtkBox, add_button: &Button) -> (GtkBox, ScrolledWindow) {
    let bar_scroll = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Automatic)
        .vscrollbar_policy(PolicyType::Never)
        .css_classes(["obsidian-tab-bar-scroller"])
        .build();
    bar_scroll.set_hexpand(true);
    bar_scroll.set_vexpand(false);
    bar_scroll.set_propagate_natural_height(true);
    bar_scroll.set_child(Some(tab_container));

    let row = GtkBox::new(Orientation::Horizontal, 8);
    row.add_css_class("obsidian-tab-bar-container");
    row.set_valign(Align::Center);
    row.append(&bar_scroll);
    row.append(add_button);
    (row, bar_scroll)
}

fn notebook() -> Notebook {
    let notebook = Notebook::new();
    notebook.set_show_tabs(false);
    notebook.set_show_border(false);
    notebook.set_hexpand(true);
    notebook.set_vexpand(true);
    notebook.add_css_class("obsidian-notebook");
    notebook
}

fn actions_box() -> (Button, Button, Button, GtkBox) {
    let actions_box = GtkBox::new(Orientation::Horizontal, 0);
    actions_box.add_css_class("obsidian-workspace-actions");
    let split_tab = action_button("view-split-left-symbolic");
    let close_tab = action_button("window-close-symbolic");
    let profile_tab = action_button("preferences-system-symbolic");
    for button in [&split_tab, &close_tab, &profile_tab] {
        actions_box.append(button);
    }
    (split_tab, close_tab, profile_tab, actions_box)
}

fn append_tab(
    tabs: &Rc<RefCell<Vec<TabView>>>,
    notebook: &Notebook,
    snapshot: TabSnapshot,
    settings: &Rc<RefCell<Settings>>,
) {
    let tab = TabView::new(snapshot, settings.clone());
    notebook.append_page(tab.root(), Some(tab.title_label()));
    tabs.borrow_mut().push(tab);
}

fn action_button(icon_name: &str) -> Button {
    Button::builder()
        .icon_name(icon_name)
        .css_classes(["obsidian-workspace-button"])
        .build()
}

fn current_index(notebook: &Notebook) -> usize {
    notebook.current_page().map(|index| index as usize).unwrap_or(0)
}

fn default_snapshot() -> WorkspaceSnapshot {
    WorkspaceSnapshot {
        active_tab: 0,
        tabs: vec![TabSnapshot {
            title: "tab 1".to_string(),
            profile: ProfileId::Default,
            left_cwd: None,
            right_cwd: None,
        }],
    }
}
