use gtk::{prelude::*, Align, Box as GtkBox, Label, Orientation};

use crate::linux_terminal::meta::APP_VERSION;

use super::widgets::body_copy;

pub(super) fn build_about_page() -> GtkBox {
    let page = GtkBox::new(Orientation::Vertical, 0);
    page.add_css_class("obsidian-settings-about-page");
    page.set_vexpand(true);
    page.set_hexpand(true);
    page.set_halign(Align::Center);
    page.set_valign(Align::Center);

    let panel = GtkBox::new(Orientation::Vertical, 12);
    panel.add_css_class("obsidian-settings-about-panel");
    panel.set_halign(Align::Center);
    panel.set_valign(Align::Start);
    panel.set_size_request(600, -1);
    panel.set_margin_top(40);
    panel.set_margin_bottom(40);

    let title = about_label("obsidian", "obsidian-settings-about-title");
    let name = about_label("X.R.1.9 - Kayode", "obsidian-settings-about-name");
    let meta = about_label(APP_VERSION, "obsidian-settings-about-meta");

    let summary = centered_copy(
        "Obsidian is a stripped terminal workspace shaped around focus, compact control, and a quiet visual system.",
    );
    
    // Credits Section
    let credits_header = section_header("Credits");
    let credits_box = GtkBox::new(Orientation::Vertical, 10);
    credits_box.add_css_class("obsidian-about-credits-box");
    
    let engine_box = GtkBox::new(Orientation::Vertical, 2);
    let engine_label = about_label("Engine", "obsidian-about-category-label");
    let engine_value = centered_copy("Rust • GTK4 • Ratatui • fontdue");
    engine_box.append(&engine_label);
    engine_box.append(&engine_value);

    let design_box = GtkBox::new(Orientation::Vertical, 2);
    let design_label = about_label("Design & Development", "obsidian-about-category-label");
    let design_value = centered_copy("Kayode (@WilliamSamson)");
    design_box.append(&design_label);
    design_box.append(&design_value);
    
    credits_box.append(&engine_box);
    credits_box.append(&design_box);

    // License Section
    let license_header = section_header("License");
    let license_text = "Copyright (c) 2026 Kayode. \n\nLicensed under the Apache License, Version 2.0 (the \"License\"); you may not use this file except in compliance with the License. You may obtain a copy of the License at:\n\nhttp://www.apache.org/licenses/LICENSE-2.0\n\nUnless required by applicable law or agreed to in writing, software distributed under the License is distributed on an \"AS IS\" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.";
    
    let license_label = Label::new(Some(license_text));
    license_label.add_css_class("obsidian-about-license-text");
    license_label.set_wrap(true);
    license_label.set_max_width_chars(60);
    license_label.set_justify(gtk::Justification::Center);

    panel.append(&title);
    panel.append(&name);
    panel.append(&meta);
    panel.append(&summary);
    
    panel.append(&credits_header);
    panel.append(&credits_box);
    
    panel.append(&license_header);
    panel.append(&license_label);

    page.append(&panel);
    page
}

fn section_header(text: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class("obsidian-about-section-header");
    label.set_margin_top(24);
    label.set_margin_bottom(8);
    label.set_halign(Align::Center);
    label
}

fn about_label(text: &str, class_name: &str) -> Label {
    let label = Label::new(Some(text));
    label.add_css_class(class_name);
    label.set_xalign(0.5);
    label.set_selectable(false);
    label
}

fn centered_copy(text: &str) -> Label {
    let copy = body_copy(text);
    copy.set_xalign(0.5);
    copy.set_justify(gtk::Justification::Center);
    copy.set_max_width_chars(72);
    copy
}
