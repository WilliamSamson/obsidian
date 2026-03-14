use std::{cell::Cell, cell::RefCell, rc::Rc};

use gtk::{
    prelude::*, Align, Box as GtkBox, Button, Image, Orientation, Overflow, PolicyType,
    Revealer, RevealerTransitionType, ScrolledWindow,
};
use webkit6::WebContext;

use super::{logr, settings::Settings, web};

const PANE_WIDTH: i32 = 420;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SidePaneKind {
    None,
    Logr,
    Web,
}

pub(super) struct SidePanes {
    pub(super) handle: GtkBox,
    pub(super) logr_revealer: Revealer,
    pub(super) web_revealer: Revealer,
}

pub(super) fn build_side_panes(
    settings: Rc<RefCell<Settings>>,
    open_on_start: bool,
) -> SidePanes {
    // Rc<Cell<SidePaneKind>> shares the currently open side pane across the segmented-handle callbacks on the GTK thread.
    let active_pane = Rc::new(Cell::new(if open_on_start {
        SidePaneKind::Logr
    } else {
        SidePaneKind::None
    }));

    let handle = GtkBox::new(Orientation::Vertical, 3);
    handle.add_css_class("obsidian-handle");
    handle.set_vexpand(false);
    handle.set_valign(Align::Center);

    let logr_button = handle_button("view-list-symbolic", "Open logr");
    let web_button = handle_button("applications-internet-symbolic", "Open web");
    handle.append(&logr_button);
    handle.append(&web_button);

    let logr_revealer = build_revealer(&wrap_pane(&logr::build_logr_pane()));
    let web_context = WebContext::new();
    let web_host = web::WebPaneHost::new(settings, web_context);
    let web_revealer = build_revealer(&wrap_pane(web_host.widget()));

    sync_side_panes(
        active_pane.get(),
        &logr_revealer,
        &web_revealer,
        &handle,
        &logr_button,
        &web_button,
        &web_host,
    );

    {
        let active_pane = active_pane.clone();
        let logr_revealer = logr_revealer.clone();
        let web_revealer = web_revealer.clone();
        let handle = handle.clone();
        let logr_button_ref = logr_button.clone();
        let web_button_ref = web_button.clone();
        let web_host_ref = web_host.clone();
        logr_button.connect_clicked(move |_| {
            let next = if active_pane.get() == SidePaneKind::Logr {
                SidePaneKind::None
            } else {
                SidePaneKind::Logr
            };
            active_pane.set(next);
            sync_side_panes(
                next,
                &logr_revealer,
                &web_revealer,
                &handle,
                &logr_button_ref,
                &web_button_ref,
                &web_host_ref,
            );
        });
    }

    {
        let active_pane = active_pane.clone();
        let logr_revealer = logr_revealer.clone();
        let web_revealer = web_revealer.clone();
        let handle = handle.clone();
        let logr_button_ref = logr_button.clone();
        let web_button_ref = web_button.clone();
        let web_host_ref = web_host.clone();
        web_button.connect_clicked(move |_| {
            let next = if active_pane.get() == SidePaneKind::Web {
                SidePaneKind::None
            } else {
                SidePaneKind::Web
            };
            active_pane.set(next);
            sync_side_panes(
                next,
                &logr_revealer,
                &web_revealer,
                &handle,
                &logr_button_ref,
                &web_button_ref,
                &web_host_ref,
            );
        });
    }

    SidePanes {
        handle,
        logr_revealer,
        web_revealer,
    }
}

fn handle_button(icon_name: &str, tooltip: &str) -> Button {
    let button = Button::builder()
        .css_classes(["obsidian-handle-segment"])
        .tooltip_text(tooltip)
        .build();
    let icon = Image::from_icon_name(icon_name);
    icon.add_css_class("obsidian-handle-icon");
    button.set_child(Some(&icon));
    button
}

fn wrap_pane(child: &impl IsA<gtk::Widget>) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.add_css_class("obsidian-right-pane");
    root.set_size_request(PANE_WIDTH, -1);
    root.set_width_request(PANE_WIDTH);
    root.set_hexpand(false);
    root.set_vexpand(true);
    root.set_valign(Align::Fill);
    root.set_overflow(Overflow::Hidden);
    root.append(child);
    root
}

fn build_revealer(child: &impl IsA<gtk::Widget>) -> Revealer {
    let revealer = Revealer::builder()
        .transition_type(RevealerTransitionType::SlideLeft)
        .transition_duration(250)
        .build();
    revealer.set_visible(false);
    revealer.set_hexpand(false);
    revealer.set_vexpand(true);
    revealer.set_halign(Align::End);
    revealer.set_width_request(PANE_WIDTH);

    let frame = ScrolledWindow::new();
    frame.set_hexpand(false);
    frame.set_vexpand(true);
    frame.set_min_content_width(PANE_WIDTH);
    frame.set_max_content_width(PANE_WIDTH);
    frame.set_propagate_natural_width(false);
    frame.set_policy(PolicyType::Never, PolicyType::Never);
    frame.set_child(Some(child));

    revealer.set_child(Some(&frame));
    revealer
}

fn sync_side_panes(
    active: SidePaneKind,
    logr_revealer: &Revealer,
    web_revealer: &Revealer,
    handle: &GtkBox,
    logr_button: &Button,
    web_button: &Button,
    web_host: &web::WebPaneHost,
) {
    let show_logr = active == SidePaneKind::Logr;
    let show_web = active == SidePaneKind::Web;

    if show_web {
        web_host.ensure_loaded();
    }

    logr_revealer.set_visible(show_logr);
    web_revealer.set_visible(show_web);
    logr_revealer.set_reveal_child(show_logr);
    web_revealer.set_reveal_child(show_web);

    set_active_button(logr_button, show_logr);
    set_active_button(web_button, show_web);

    if active == SidePaneKind::None {
        handle.add_css_class("collapsed");
    } else {
        handle.remove_css_class("collapsed");
    }
}

fn set_active_button(button: &Button, active: bool) {
    if active {
        button.add_css_class("active");
    } else {
        button.remove_css_class("active");
    }
}
