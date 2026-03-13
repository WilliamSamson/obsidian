use gtk::{gdk, pango::FontDescription};
use vte4::{prelude::*, CursorBlinkMode, CursorShape, Terminal};

use super::profile::{profile, ProfileId};

const SCROLLBACK_LINES: i64 = 20_000;

pub(super) fn build_terminal(profile_id: ProfileId) -> Terminal {
    let terminal = Terminal::builder()
        .hexpand(true)
        .vexpand(true)
        .can_focus(true)
        .focus_on_click(true)
        .focusable(true)
        .scrollback_lines(SCROLLBACK_LINES as u32)
        .allow_hyperlink(true)
        .build();
    terminal.add_css_class("obsidian-terminal");
    terminal.set_cursor_blink_mode(CursorBlinkMode::Off);
    terminal.set_cursor_shape(CursorShape::Ibeam);
    terminal.set_font(Some(&FontDescription::from_string("DejaVu Sans Mono 10")));
    terminal.set_font_scale(profile(profile_id).font_scale);
    let palette = [
        rgba(0.00, 0.00, 0.00),
        rgba(1.00, 0.20, 0.20),
        rgba(0.28, 0.66, 0.35),
        rgba(0.96, 0.80, 0.00),
        rgba(0.92, 0.92, 0.92),
        rgba(0.70, 0.70, 0.70),
        rgba(0.70, 0.70, 0.70),
        rgba(0.96, 0.96, 0.96),
    ];
    let palette_refs = palette.iter().collect::<Vec<_>>();
    terminal.set_colors(
        Some(&rgba(0.96, 0.96, 0.96)),
        Some(&rgba(0.00, 0.00, 0.00)),
        &palette_refs,
    );
    terminal
}

fn rgba(red: f32, green: f32, blue: f32) -> gdk::RGBA {
    gdk::RGBA::new(red, green, blue, 1.0)
}
