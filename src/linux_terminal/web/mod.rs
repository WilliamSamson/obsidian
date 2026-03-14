mod browser;
mod host;
mod navigation;
use std::{cell::RefCell, rc::Rc};
use gtk::{
    pango, prelude::*, Box as GtkBox, Button, Entry, Label, Orientation, Overflow, PolicyType,
    ScrolledWindow,
};
use webkit6::{Settings as WebSettings, WebContext, WebView};
use super::settings::Settings;
use browser::load_home_page;
pub(super) use host::WebPaneHost;
use navigation::bind_navigation;

pub(super) fn build_web_pane(settings: Rc<RefCell<Settings>>, context: WebContext) -> GtkBox {
    let root = GtkBox::new(Orientation::Vertical, 0);
    root.set_vexpand(true);
    root.set_hexpand(true);
    root.set_width_request(0);
    root.set_overflow(Overflow::Hidden);
    root.add_css_class("obsidian-web-root");

    let controls = build_controls();
    let status = build_status();
    let (frame, web_view) = build_web_frame(&context);

    root.append(&controls.row);
    root.append(&status);
    root.append(&frame);

    let state = Rc::new(WebPaneState {
        settings,
        web_view,
        address: controls.address,
        status,
        back_button: controls.back_button,
        forward_button: controls.forward_button,
    });

    bind_navigation(
        &state,
        &controls.reload_button,
        &controls.home_button,
        &controls.zoom_out_button,
        &controls.zoom_reset_button,
        &controls.go_button,
    );
    load_home_page(&state);

    root
}

pub(super) struct WebPaneState {
    pub(super) settings: Rc<RefCell<Settings>>,
    pub(super) web_view: WebView,
    pub(super) address: Entry,
    pub(super) status: Label,
    pub(super) back_button: Button,
    pub(super) forward_button: Button,
}

struct ControlWidgets {
    row: GtkBox,
    back_button: Button,
    forward_button: Button,
    reload_button: Button,
    home_button: Button,
    zoom_out_button: Button,
    zoom_reset_button: Button,
    go_button: Button,
    address: Entry,
}

fn build_controls() -> ControlWidgets {
    let row = GtkBox::new(Orientation::Horizontal, 0);
    row.add_css_class("obsidian-web-bar");
    row.add_css_class("obsidian-web-controls");
    row.set_hexpand(true);
    row.set_overflow(Overflow::Hidden);

    let nav = GtkBox::new(Orientation::Horizontal, 2);
    nav.add_css_class("obsidian-web-nav");

    let back_button = icon_button("go-previous-symbolic", "Back");
    let forward_button = icon_button("go-next-symbolic", "Forward");
    let reload_button = icon_button("view-refresh-symbolic", "Reload");
    let home_button = icon_button("go-home-symbolic", "Home");
    let zoom_out_button = icon_button("zoom-out-symbolic", "Zoom out");
    let zoom_reset_button = text_button("100%", "Reset zoom");
    let go_button = icon_button("go-jump-symbolic", "Open");

    nav.append(&back_button);
    nav.append(&forward_button);
    nav.append(&reload_button);
    nav.append(&home_button);
    nav.append(&zoom_out_button);
    nav.append(&zoom_reset_button);

    let address = Entry::new();
    address.add_css_class("obsidian-web-entry");
    address.set_placeholder_text(Some("search or enter address"));
    address.set_hexpand(true);
    address.set_width_request(0);

    let address_shell = GtkBox::new(Orientation::Horizontal, 0);
    address_shell.add_css_class("obsidian-web-address-shell");
    address_shell.set_hexpand(true);
    address_shell.set_overflow(Overflow::Hidden);
    address_shell.append(&address);
    address_shell.append(&go_button);

    row.append(&nav);
    row.append(&address_shell);

    ControlWidgets {
        row,
        back_button,
        forward_button,
        reload_button,
        home_button,
        zoom_out_button,
        zoom_reset_button,
        go_button,
        address,
    }
}

fn build_status() -> Label {
    let status = Label::new(Some("ready"));
    status.add_css_class("obsidian-web-status");
    status.set_xalign(0.0);
    status.set_ellipsize(pango::EllipsizeMode::End);
    status
}

fn build_web_frame(context: &WebContext) -> (ScrolledWindow, WebView) {
    let web_settings = WebSettings::new();
    web_settings.set_enable_developer_extras(true);
    web_settings.set_enable_back_forward_navigation_gestures(true);
    web_settings.set_enable_smooth_scrolling(true);

    let web_view = WebView::builder()
        .web_context(context)
        .settings(&web_settings)
        .zoom_level(0.75)
        .hexpand(true)
        .vexpand(true)
        .build();
    web_view.add_css_class("obsidian-webview");
    web_view.set_hexpand(false);
    web_view.set_vexpand(true);
    web_view.set_width_request(0);
    web_view.set_overflow(Overflow::Hidden);

    let frame = ScrolledWindow::new();
    frame.add_css_class("obsidian-web-frame");
    frame.set_hexpand(true);
    frame.set_vexpand(true);
    frame.set_min_content_width(0);
    frame.set_max_content_width(392);
    frame.set_propagate_natural_width(false);
    frame.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    frame.set_vscrollbar_policy(PolicyType::Automatic);
    frame.set_width_request(0);
    frame.set_overflow(Overflow::Hidden);
    frame.set_child(Some(&web_view));

    (frame, web_view)
}

fn icon_button(icon_name: &str, tooltip: &str) -> Button {
    Button::builder()
        .icon_name(icon_name)
        .tooltip_text(tooltip)
        .css_classes(["obsidian-web-button"])
        .build()
}

fn text_button(label: &str, tooltip: &str) -> Button {
    Button::builder()
        .label(label)
        .tooltip_text(tooltip)
        .css_classes(["obsidian-web-button", "obsidian-web-text-button"])
        .build()
}
