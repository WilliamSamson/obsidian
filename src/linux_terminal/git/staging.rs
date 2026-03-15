use std::rc::Rc;

use gtk::{
    pango, prelude::*, Box as GtkBox, Button, Label, ListBox, Orientation,
    PolicyType, Revealer, RevealerTransitionType, ScrolledWindow, SelectionMode, TextView,
};

use super::{
    diff::build_diff_widget,
    ops::{self, FileChange, FileStatus, RepoStatus},
    GitPaneView,
};

pub(super) struct StagingWidgets {
    pub staged_list: ListBox,
    pub unstaged_list: ListBox,
    pub untracked_list: ListBox,
    pub staged_count: Label,
    pub unstaged_count: Label,
    pub untracked_count: Label,
    pub commit_entry: TextView,
    pub commit_button: Button,
}

pub(super) fn build_staging_view(view: &Rc<GitPaneView>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("obsidian-git-staging-root");
    root.set_vexpand(true);

    // Commit message area
    let commit_box = GtkBox::new(Orientation::Vertical, 4);
    commit_box.add_css_class("obsidian-git-commit-box");

    let commit_entry = TextView::new();
    commit_entry.add_css_class("obsidian-git-commit-entry");
    commit_entry.set_wrap_mode(gtk::WrapMode::WordChar);
    commit_entry.set_accepts_tab(false);
    commit_entry.set_top_margin(6);
    commit_entry.set_bottom_margin(6);
    commit_entry.set_left_margin(8);
    commit_entry.set_right_margin(8);

    let buffer = commit_entry.buffer();

    let commit_scroller = ScrolledWindow::new();
    commit_scroller.set_child(Some(&commit_entry));
    commit_scroller.set_min_content_height(60);
    commit_scroller.set_max_content_height(100);
    commit_scroller.set_policy(PolicyType::Never, PolicyType::Automatic);

    let commit_actions = GtkBox::new(Orientation::Horizontal, 4);
    commit_actions.add_css_class("obsidian-git-commit-actions");

    let commit_button = Button::builder()
        .label("commit")
        .css_classes(["obsidian-git-commit-button"])
        .sensitive(false)
        .build();

    let stage_all_button = Button::builder()
        .label("stage all")
        .css_classes(["obsidian-git-action-btn"])
        .build();

    let unstage_all_button = Button::builder()
        .label("unstage all")
        .css_classes(["obsidian-git-action-btn"])
        .build();

    commit_actions.append(&stage_all_button);
    commit_actions.append(&unstage_all_button);

    let spacer = GtkBox::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    commit_actions.append(&spacer);
    commit_actions.append(&commit_button);

    commit_box.append(&commit_scroller);
    commit_box.append(&commit_actions);

    // File sections
    let (staged_section, staged_list, staged_count) = build_file_section("staged");
    let (unstaged_section, unstaged_list, unstaged_count) = build_file_section("unstaged");
    let (untracked_section, untracked_list, untracked_count) = build_file_section("untracked");

    let files_scroller = ScrolledWindow::new();
    files_scroller.set_vexpand(true);
    files_scroller.set_policy(PolicyType::Never, PolicyType::Automatic);

    let files_box = GtkBox::new(Orientation::Vertical, 0);
    files_box.append(&staged_section);
    files_box.append(&unstaged_section);
    files_box.append(&untracked_section);
    files_scroller.set_child(Some(&files_box));

    root.append(&commit_box);
    root.append(&files_scroller);

    // Store widgets
    let staging_widgets = Rc::new(StagingWidgets {
        staged_list,
        unstaged_list,
        untracked_list,
        staged_count,
        unstaged_count,
        untracked_count,
        commit_entry: commit_entry.clone(),
        commit_button: commit_button.clone(),
    });

    *view.staging_widgets.borrow_mut() = Some(staging_widgets.clone());

    // Bind commit
    {
        let view_ref = view.clone();
        let entry = commit_entry.clone();
        commit_button.connect_clicked(move |_| {
            let buf = entry.buffer();
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
            let message = text.trim().to_string();
            if message.is_empty() {
                return;
            }
            let repo = view_ref.repo_root.borrow().clone();
            if let Some(repo) = repo {
                match ops::git_commit(&repo, &message) {
                    Ok(_) => {
                        buf.set_text("");
                        view_ref.set_status("committed");
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status(&format!("commit failed: {e}")),
                }
            }
        });
    }

    // Enable commit button when there's text
    {
        let btn = commit_button.clone();
        buffer.connect_changed(move |buf| {
            let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
            btn.set_sensitive(!text.trim().is_empty());
        });
    }

    // Stage all
    {
        let view_ref = view.clone();
        stage_all_button.connect_clicked(move |_| {
            let repo = view_ref.repo_root.borrow().clone();
            if let Some(repo) = repo {
                match ops::git_stage_all(&repo) {
                    Ok(_) => {
                        view_ref.set_status("staged all");
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status(&format!("stage all failed: {e}")),
                }
            }
        });
    }

    // Unstage all
    {
        let view_ref = view.clone();
        unstage_all_button.connect_clicked(move |_| {
            let repo = view_ref.repo_root.borrow().clone();
            if let Some(repo) = repo {
                match ops::git_unstage_all(&repo) {
                    Ok(_) => {
                        view_ref.set_status("unstaged all");
                        view_ref.refresh();
                    }
                    Err(e) => view_ref.set_status(&format!("unstage all failed: {e}")),
                }
            }
        });
    }

    root
}

pub(super) fn refresh_staging(view: &Rc<GitPaneView>, status: &RepoStatus) {
    let widgets = view.staging_widgets.borrow();
    let Some(widgets) = widgets.as_ref() else {
        return;
    };

    // Staged files
    clear_list(&widgets.staged_list);
    widgets.staged_count.set_text(&status.staged.len().to_string());
    for file in &status.staged {
        widgets.staged_list.append(&build_file_row(file, true, view));
    }

    // Unstaged files
    clear_list(&widgets.unstaged_list);
    widgets.unstaged_count.set_text(&status.unstaged.len().to_string());
    for file in &status.unstaged {
        widgets.unstaged_list.append(&build_file_row(file, false, view));
    }

    // Untracked files
    clear_list(&widgets.untracked_list);
    widgets.untracked_count.set_text(&status.untracked.len().to_string());
    for path in &status.untracked {
        widgets.untracked_list.append(&build_untracked_row(path, view));
    }

    // Enable commit button only if staged changes exist
    let has_staged = !status.staged.is_empty();
    let buf = widgets.commit_entry.buffer();
    let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
    widgets.commit_button.set_sensitive(has_staged && !text.trim().is_empty());
}

// ─── File section builder ────────────────────────────────────────────

fn build_file_section(title: &str) -> (GtkBox, ListBox, Label) {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("obsidian-git-file-section");

    let header = GtkBox::new(Orientation::Horizontal, 4);
    header.add_css_class("obsidian-git-file-section-header");

    let arrow = Label::new(Some("\u{25BC}"));
    arrow.add_css_class("obsidian-git-section-arrow");

    let title_label = Label::new(Some(title));
    title_label.add_css_class("obsidian-git-section-title");
    title_label.set_hexpand(true);
    title_label.set_xalign(0.0);

    let count = Label::new(Some("0"));
    count.add_css_class("obsidian-git-section-count");

    header.append(&arrow);
    header.append(&title_label);
    header.append(&count);

    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .transition_duration(200)
        .reveal_child(true)
        .build();

    let list = ListBox::new();
    list.set_selection_mode(SelectionMode::None);
    list.add_css_class("obsidian-git-file-list");
    revealer.set_child(Some(&list));

    // Toggle on header click
    let revealer_ref = revealer.clone();
    let arrow_ref = arrow.clone();
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(move |_, _, _, _| {
        let expanded = revealer_ref.reveals_child();
        revealer_ref.set_reveal_child(!expanded);
        arrow_ref.set_text(if expanded { "\u{25B6}" } else { "\u{25BC}" });
    });
    header.add_controller(gesture);

    root.append(&header);
    root.append(&revealer);

    (root, list, count)
}

// ─── File row builders ───────────────────────────────────────────────

fn build_file_row(file: &FileStatus, is_staged: bool, view: &Rc<GitPaneView>) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.add_css_class("obsidian-git-file-row-container");

    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("obsidian-git-file-row");

    let badge = Label::new(Some(file.status.label()));
    badge.add_css_class("obsidian-git-file-status");
    badge.add_css_class(match file.status {
        FileChange::Added => "status-added",
        FileChange::Modified => "status-modified",
        FileChange::Deleted => "status-deleted",
        _ => "status-other",
    });

    let name = Label::new(Some(&file.path));
    name.add_css_class("obsidian-git-file-name");
    name.set_xalign(0.0);
    name.set_hexpand(true);
    name.set_ellipsize(pango::EllipsizeMode::Start);

    let action_label = if is_staged { "unstage" } else { "stage" };
    let action_btn = Button::builder()
        .label(action_label)
        .css_classes(["obsidian-git-file-action"])
        .build();

    let path = file.path.clone();
    let view_ref = view.clone();
    action_btn.connect_clicked(move |_| {
        let repo = view_ref.repo_root.borrow().clone();
        if let Some(repo) = repo {
            let result = if is_staged {
                ops::git_unstage_file(&repo, &path)
            } else {
                ops::git_stage_file(&repo, &path)
            };
            match result {
                Ok(_) => view_ref.refresh(),
                Err(e) => view_ref.set_status(&format!("failed: {e}")),
            }
        }
    });

    row.append(&badge);
    row.append(&name);
    row.append(&action_btn);

    // Discard button for unstaged changes
    if !is_staged && file.status != FileChange::Added {
        let discard_btn = Button::builder()
            .icon_name("edit-delete-symbolic")
            .css_classes(["obsidian-git-file-discard"])
            .tooltip_text("Discard changes")
            .build();

        let path = file.path.clone();
        let view_ref = view.clone();
        discard_btn.connect_clicked(move |_| {
            let repo = view_ref.repo_root.borrow().clone();
            if let Some(repo) = repo {
                match ops::git_discard_file(&repo, &path) {
                    Ok(_) => view_ref.refresh(),
                    Err(e) => view_ref.set_status(&format!("discard failed: {e}")),
                }
            }
        });
        row.append(&discard_btn);
    }

    // Expandable diff
    let diff_revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideDown)
        .transition_duration(200)
        .build();

    let file_path = file.path.clone();
    let view_ref = view.clone();
    let revealer_ref = diff_revealer.clone();
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(move |_, _, _, _| {
        if revealer_ref.reveals_child() {
            revealer_ref.set_reveal_child(false);
            return;
        }
        let repo = view_ref.repo_root.borrow().clone();
        if let Some(repo) = repo {
            if let Ok(hunks) = ops::git_diff_file(&repo, &file_path, is_staged) {
                let diff_widget = build_diff_widget(&hunks, None);
                revealer_ref.set_child(Some(&diff_widget));
                revealer_ref.set_reveal_child(true);
            }
        }
    });
    row.add_controller(gesture);

    container.append(&row);
    container.append(&diff_revealer);
    container
}

fn build_untracked_row(path: &str, view: &Rc<GitPaneView>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 6);
    row.add_css_class("obsidian-git-file-row");

    let badge = Label::new(Some("?"));
    badge.add_css_class("obsidian-git-file-status");
    badge.add_css_class("status-other");

    let name = Label::new(Some(path));
    name.add_css_class("obsidian-git-file-name");
    name.set_xalign(0.0);
    name.set_hexpand(true);
    name.set_ellipsize(pango::EllipsizeMode::Start);

    let stage_btn = Button::builder()
        .label("stage")
        .css_classes(["obsidian-git-file-action"])
        .build();

    let file_path = path.to_string();
    let view_ref = view.clone();
    stage_btn.connect_clicked(move |_| {
        let repo = view_ref.repo_root.borrow().clone();
        if let Some(repo) = repo {
            match ops::git_stage_file(&repo, &file_path) {
                Ok(_) => view_ref.refresh(),
                Err(e) => view_ref.set_status(&format!("stage failed: {e}")),
            }
        }
    });

    row.append(&badge);
    row.append(&name);
    row.append(&stage_btn);
    row
}

fn clear_list(list: &ListBox) {
    while let Some(child) = list.first_child() {
        list.remove(&child);
    }
}
