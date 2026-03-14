use gtk::{
    prelude::*, Align, Box as GtkBox, Button, DropDown, Entry, Label, Orientation, SpinButton,
    StringList, Switch,
};

pub(super) fn section_label(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("obsidian-settings-section");
    label.set_xalign(0.0);
    label
}

pub(super) fn text_row(parent: &GtkBox, label_text: &str, value: &str) -> Entry {
    let row = setting_row(parent, label_text);
    let entry = Entry::new();
    entry.set_text(value);
    entry.add_css_class("obsidian-settings-entry");
    entry.set_width_chars(20);
    entry.set_halign(Align::End);
    row.append(&entry);
    entry
}

pub(super) fn spin_row(
    parent: &GtkBox,
    label_text: &str,
    value: f64,
    min: f64,
    max: f64,
) -> SpinButton {
    let row = setting_row(parent, label_text);
    let spin = SpinButton::with_range(min, max, 1.0);
    spin.set_value(value);
    spin.add_css_class("obsidian-settings-spin");
    spin.set_halign(Align::End);
    row.append(&spin);
    spin
}

pub(super) fn switch_row(parent: &GtkBox, label_text: &str, active: bool) -> Switch {
    let row = setting_row(parent, label_text);
    let switch = Switch::new();
    switch.set_active(active);
    switch.add_css_class("obsidian-settings-switch");
    switch.set_halign(Align::End);
    switch.set_valign(Align::Center);
    row.append(&switch);
    switch
}

pub(super) fn dropdown_row(
    parent: &GtkBox,
    label_text: &str,
    items: &[&str],
    active: u32,
) -> DropDown {
    let row = setting_row(parent, label_text);
    let model = StringList::new(items);
    let dropdown = DropDown::builder().model(&model).selected(active).build();
    dropdown.add_css_class("obsidian-settings-dropdown");
    dropdown.set_halign(Align::End);
    row.append(&dropdown);
    dropdown
}

pub(super) fn info_row(parent: &GtkBox, label_text: &str, value: &str) {
    let row = setting_row(parent, label_text);
    let value_label = value_label(value);
    value_label.set_xalign(1.0);
    value_label.set_halign(Align::End);
    row.append(&value_label);
}

pub(super) fn action_row(parent: &GtkBox, label_text: &str, action_text: &str) -> Button {
    let row = setting_row(parent, label_text);
    let button = Button::builder()
        .label(action_text)
        .css_classes(["obsidian-settings-link"])
        .build();
    button.set_halign(Align::End);
    row.append(&button);
    button
}

pub(super) fn body_copy(text: &str) -> Label {
    let copy = Label::new(Some(text));
    copy.add_css_class("obsidian-settings-about-copy");
    copy.set_xalign(0.0);
    copy.set_wrap(true);
    copy.set_selectable(false);
    copy
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

fn value_label(value: &str) -> Label {
    let label = Label::new(Some(value));
    label.add_css_class("obsidian-settings-value");
    label.set_wrap(true);
    label.set_selectable(false);
    label
}
