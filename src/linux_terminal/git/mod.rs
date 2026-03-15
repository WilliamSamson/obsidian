#[allow(dead_code)]
mod branches;
#[allow(dead_code)]
mod diff;
#[allow(dead_code)]
mod graph;
mod host;
#[allow(dead_code)]
mod ops;
#[allow(dead_code)]
mod search;
#[allow(dead_code)]
mod staging;
#[allow(dead_code)]
mod stash;

use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

use gtk::{
    gdk, glib, prelude::*, Box as GtkBox, Button, EventControllerKey, Label, Orientation,
    Stack, StackTransitionType,
};

pub(super) use host::GitPaneHost;

use super::view::CwdProvider;

// ─── Sub-view identifiers ────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SubView {
    Status,
    Log,
    Branches,
    Stash,
    Search,
}

impl SubView {
    fn stack_name(&self) -> &'static str {
        match self {
            SubView::Status => "status",
            SubView::Log => "log",
            SubView::Branches => "branches",
            SubView::Stash => "stash",
            SubView::Search => "search",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            SubView::Status => "status",
            SubView::Log => "log",
            SubView::Branches => "branches",
            SubView::Stash => "stash",
            SubView::Search => "search",
        }
    }
}

const SUB_VIEWS: &[SubView] = &[
    SubView::Status,
    SubView::Log,
    SubView::Branches,
    SubView::Stash,
    SubView::Search,
];

// ─── Shared pane state ───────────────────────────────────────────────

pub(super) struct GitPaneView {
    root: GtkBox,
    cwd_provider: CwdProvider,
    repo_root: RefCell<Option<PathBuf>>,
    branch_label: Label,
    ahead_behind_label: Label,
    status_label: Label,
    nav_stack: Stack,
    active_view: RefCell<SubView>,
    nav_buttons: Vec<Button>,

    // Remote action buttons
    fetch_btn: Button,
    _pull_btn: Button,
    _push_btn: Button,

    // Sub-view widget refs (populated by each sub-view builder)
    staging_widgets: RefCell<Option<Rc<staging::StagingWidgets>>>,
    graph_widgets: RefCell<Option<Rc<graph::GraphState>>>,
    branch_widgets: RefCell<Option<branches::BranchWidgets>>,
    stash_widgets: RefCell<Option<stash::StashWidgets>>,
    search_widgets: RefCell<Option<Rc<search::SearchWidgets>>>,
}

impl GitPaneView {
    fn set_status(&self, text: &str) {
        self.status_label.set_text(text);
    }

    fn switch_view(&self, view: SubView) {
        *self.active_view.borrow_mut() = view;
        self.nav_stack.set_visible_child_name(view.stack_name());

        for (i, btn) in self.nav_buttons.iter().enumerate() {
            if SUB_VIEWS[i] == view {
                btn.add_css_class("active");
            } else {
                btn.remove_css_class("active");
            }
        }
    }
}

// Free function so closures can capture Rc<GitPaneView> and call it.
fn refresh(view: &Rc<GitPaneView>) {
    let cwd = (view.cwd_provider)().map(PathBuf::from)
        .or_else(|| std::env::current_dir().ok());

    let Some(cwd) = cwd else {
        set_no_repo(view);
        return;
    };

    match ops::git_repo_root(&cwd) {
        Ok(root) => {
            *view.repo_root.borrow_mut() = Some(root.clone());

            match ops::git_status(&root) {
                Ok(status) => {
                    view.branch_label.set_text(&status.branch);

                    let ab = format_ahead_behind(status.ahead, status.behind);
                    view.ahead_behind_label.set_text(&ab);
                    view.ahead_behind_label.set_visible(!ab.is_empty());

                    let active = *view.active_view.borrow();
                    match active {
                        SubView::Status => staging::refresh_staging(view, &status),
                        SubView::Log => graph::refresh_graph(view),
                        SubView::Branches => branches::refresh_branches(view),
                        SubView::Stash => stash::refresh_stash(view),
                        SubView::Search => {} // search refreshes on demand
                    }
                }
                Err(e) => view.set_status(&format!("status error: {e}")),
            }
        }
        Err(_) => set_no_repo(view),
    }
}

fn set_no_repo(view: &Rc<GitPaneView>) {
    *view.repo_root.borrow_mut() = None;
    view.branch_label.set_text("not a git repo");
    view.ahead_behind_label.set_visible(false);
    view.set_status("no git repository found");
}

// Convenience method for closures that already hold Rc<GitPaneView>
impl GitPaneView {
    fn refresh(self: &Rc<Self>) {
        refresh(self);
    }
}

// ─── Public entry point ──────────────────────────────────────────────

pub(super) fn build_git_pane(cwd_provider: CwdProvider) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.add_css_class("obsidian-git-root");
    root.set_focusable(true);

    // ─── Header: branch + ahead/behind ───────────────────────────
    let header = GtkBox::new(Orientation::Horizontal, 6);
    header.add_css_class("obsidian-git-header");

    let title = Label::new(Some("git"));
    title.add_css_class("obsidian-git-title");

    let branch_label = Label::new(Some("..."));
    branch_label.add_css_class("obsidian-git-branch-label");
    branch_label.set_xalign(0.0);
    branch_label.set_hexpand(true);
    branch_label.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let ahead_behind_label = Label::new(None);
    ahead_behind_label.add_css_class("obsidian-git-ahead-behind");
    ahead_behind_label.set_visible(false);

    header.append(&title);
    header.append(&branch_label);
    header.append(&ahead_behind_label);

    // ─── Remote operations bar ───────────────────────────────────
    let remote_bar = GtkBox::new(Orientation::Horizontal, 4);
    remote_bar.add_css_class("obsidian-git-remote-bar");

    let fetch_btn = Button::builder()
        .label("fetch")
        .css_classes(["obsidian-git-remote-button"])
        .tooltip_text("Fetch all remotes")
        .build();

    let pull_btn = Button::builder()
        .label("pull")
        .css_classes(["obsidian-git-remote-button"])
        .tooltip_text("Pull from remote")
        .build();

    let push_btn = Button::builder()
        .label("push")
        .css_classes(["obsidian-git-remote-button"])
        .tooltip_text("Push to remote")
        .build();

    let refresh_btn = Button::builder()
        .icon_name("view-refresh-symbolic")
        .css_classes(["obsidian-git-icon-btn"])
        .tooltip_text("Refresh (Ctrl+R)")
        .build();

    remote_bar.append(&fetch_btn);
    remote_bar.append(&pull_btn);
    remote_bar.append(&push_btn);
    let remote_spacer = GtkBox::new(Orientation::Horizontal, 0);
    remote_spacer.set_hexpand(true);
    remote_bar.append(&remote_spacer);
    remote_bar.append(&refresh_btn);

    // ─── Sub-view navigation ─────────────────────────────────────
    let nav_row = GtkBox::new(Orientation::Horizontal, 2);
    nav_row.add_css_class("obsidian-git-nav");

    let mut nav_buttons = Vec::new();
    for sv in SUB_VIEWS {
        let btn = Button::builder()
            .label(sv.label())
            .css_classes(["obsidian-git-nav-button"])
            .build();
        if *sv == SubView::Status {
            btn.add_css_class("active");
        }
        nav_row.append(&btn);
        nav_buttons.push(btn);
    }

    // ─── Content stack ───────────────────────────────────────────
    let nav_stack = Stack::new();
    nav_stack.set_transition_type(StackTransitionType::Crossfade);
    nav_stack.set_transition_duration(150);
    nav_stack.set_vexpand(true);

    // ─── Status bar ──────────────────────────────────────────────
    let status_label = Label::new(Some("loading..."));
    status_label.add_css_class("obsidian-git-status");
    status_label.set_xalign(0.0);
    status_label.set_ellipsize(gtk::pango::EllipsizeMode::End);

    // ─── Assemble ────────────────────────────────────────────────
    root.append(&header);
    root.append(&remote_bar);
    root.append(&nav_row);
    root.append(&nav_stack);
    root.append(&status_label);

    let view = Rc::new(GitPaneView {
        root: root.clone(),
        cwd_provider,
        repo_root: RefCell::new(None),
        branch_label,
        ahead_behind_label,
        status_label,
        nav_stack: nav_stack.clone(),
        active_view: RefCell::new(SubView::Status),
        nav_buttons: nav_buttons.clone(),
        fetch_btn: fetch_btn.clone(),
        _pull_btn: pull_btn.clone(),
        _push_btn: push_btn.clone(),
        staging_widgets: RefCell::new(None),
        graph_widgets: RefCell::new(None),
        branch_widgets: RefCell::new(None),
        stash_widgets: RefCell::new(None),
        search_widgets: RefCell::new(None),
    });

    // Build sub-views and add to stack
    let status_view = staging::build_staging_view(&view);
    let log_view = graph::build_graph_view(&view);
    let branches_view = branches::build_branches_view(&view);
    let stash_view = stash::build_stash_view(&view);
    let search_view = search::build_search_view(&view);

    nav_stack.add_named(&status_view, Some("status"));
    nav_stack.add_named(&log_view, Some("log"));
    nav_stack.add_named(&branches_view, Some("branches"));
    nav_stack.add_named(&stash_view, Some("stash"));
    nav_stack.add_named(&search_view, Some("search"));

    // ─── Bind navigation buttons ─────────────────────────────────
    for (i, btn) in nav_buttons.iter().enumerate() {
        let view_ref = view.clone();
        let sv = SUB_VIEWS[i];
        btn.connect_clicked(move |_| {
            view_ref.switch_view(sv);
            view_ref.refresh();
        });
    }

    // ─── Bind remote operations ──────────────────────────────────
    {
        let view_ref = view.clone();
        fetch_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            let repo = view_ref.repo_root.borrow().clone();
            if let Some(repo) = repo {
                view_ref.set_status("fetching...");
                match ops::git_fetch(&repo) {
                    Ok(_) => {
                        view_ref.set_status("fetch complete");
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status(&format!("fetch failed: {e}")),
                }
            }
            btn.set_sensitive(true);
        });
    }

    {
        let view_ref = view.clone();
        pull_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            let repo = view_ref.repo_root.borrow().clone();
            if let Some(repo) = repo {
                view_ref.set_status("pulling...");
                match ops::git_pull(&repo) {
                    Ok(output) => {
                        let summary = output.lines().next().unwrap_or("pull complete");
                        view_ref.set_status(summary);
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status(&format!("pull failed: {e}")),
                }
            }
            btn.set_sensitive(true);
        });
    }

    {
        let view_ref = view.clone();
        push_btn.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            let repo = view_ref.repo_root.borrow().clone();
            if let Some(repo) = repo {
                view_ref.set_status("pushing...");
                match ops::git_push(&repo) {
                    Ok(_) => {
                        view_ref.set_status("push complete");
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status(&format!("push failed: {e}")),
                }
            }
            btn.set_sensitive(true);
        });
    }

    // ─── Bind refresh button ─────────────────────────────────────
    {
        let view_ref = view.clone();
        refresh_btn.connect_clicked(move |_| {
            view_ref.refresh();
        });
    }

    // ─── Keyboard shortcuts ──────────────────────────────────────
    bind_keyboard(&view);

    // ─── Initial load + auto-refresh ─────────────────────────────
    {
        let view_ref = view.clone();
        glib::idle_add_local_once(move || {
            view_ref.refresh();
        });
    }

    watch_directory(&view);

    root
}

// ─── Keyboard ────────────────────────────────────────────────────────

fn bind_keyboard(view: &Rc<GitPaneView>) {
    let key_ctrl = EventControllerKey::new();
    let view_ref = view.clone();

    key_ctrl.connect_key_pressed(move |_, keyval, _keycode, modifier| {
        let ctrl = modifier.contains(gdk::ModifierType::CONTROL_MASK);

        // Ctrl+R → refresh
        if ctrl && keyval == gdk::Key::r {
            view_ref.refresh();
            return glib::Propagation::Stop;
        }

        // Ctrl+F → fetch
        if ctrl && keyval == gdk::Key::f {
            view_ref.fetch_btn.emit_clicked();
            return glib::Propagation::Stop;
        }

        // 1-5 → switch sub-views
        if !ctrl {
            let sv = match keyval {
                gdk::Key::_1 => Some(SubView::Status),
                gdk::Key::_2 => Some(SubView::Log),
                gdk::Key::_3 => Some(SubView::Branches),
                gdk::Key::_4 => Some(SubView::Stash),
                gdk::Key::_5 => Some(SubView::Search),
                _ => None,
            };
            if let Some(sv) = sv {
                view_ref.switch_view(sv);
                view_ref.refresh();
                return glib::Propagation::Stop;
            }
        }

        // Escape → return to status
        if keyval == gdk::Key::Escape {
            view_ref.switch_view(SubView::Status);
            view_ref.refresh();
            return glib::Propagation::Stop;
        }

        glib::Propagation::Proceed
    });

    view.root.add_controller(key_ctrl);
}

// ─── Auto-refresh on directory change ────────────────────────────────

fn watch_directory(view: &Rc<GitPaneView>) {
    let view_ref = view.clone();
    let last_cwd: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    glib::timeout_add_local(Duration::from_millis(2000), move || {
        let current = (view_ref.cwd_provider)();
        let changed = {
            let last = last_cwd.borrow();
            *last != current
        };

        if changed {
            *last_cwd.borrow_mut() = current;
            view_ref.refresh();
        }

        glib::ControlFlow::Continue
    });
}

// ─── Helpers ─────────────────────────────────────────────────────────

fn format_ahead_behind(ahead: u32, behind: u32) -> String {
    match (ahead, behind) {
        (0, 0) => String::new(),
        (a, 0) => format!("\u{2191}{a}"),
        (0, b) => format!("\u{2193}{b}"),
        (a, b) => format!("\u{2191}{a} \u{2193}{b}"),
    }
}
