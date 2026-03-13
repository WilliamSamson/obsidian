use gtk::{prelude::*, Button, HeaderBar, Image, Label};

use super::{APP_TITLE, HEADER_ICON_PATH};

pub(super) fn build_header() -> (HeaderBar, Button) {
    let header = HeaderBar::new();
    header.add_css_class("obsidian-header");
    header.set_show_title_buttons(true);
    header.set_decoration_layout(Some(":minimize,maximize,close"));

    let logo = Image::from_file(HEADER_ICON_PATH);
    logo.set_pixel_size(16);
    logo.add_css_class("obsidian-logo");
    header.pack_start(&logo);

    let title = Label::new(Some(APP_TITLE));
    title.add_css_class("obsidian-title");
    header.set_title_widget(Some(&title));

    let settings_button = Button::builder()
        .icon_name("emblem-system-symbolic")
        .css_classes(["obsidian-header-settings"])
        .tooltip_text("Settings")
        .build();
    header.pack_end(&settings_button);

    (header, settings_button)
}
