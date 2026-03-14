use gtk::{
    gdk, style_context_add_provider_for_display, CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION,
};

use crate::ui::theme;

pub(super) fn install_css() {
    let provider = CssProvider::new();
    let css = format!(
        "
        window.obsidian-window {{
            background: {window_bg};
            border: 1px solid {window_edge};
            border-radius: 12px;
        }}

        headerbar.obsidian-header {{
            background: {titlebar_bg};
            border-bottom: 1px solid {border};
            min-height: 40px;
            padding: 4px 12px;
        }}

        headerbar.obsidian-header box {{
            background: transparent;
        }}

        headerbar.obsidian-header button.titlebutton {{
            background: {surface};
            border-radius: 50%;
            box-shadow: none;
            color: transparent;
            min-height: 14px;
            min-width: 14px;
            padding: 0;
            margin: 0 4px;
            transition: background 140ms ease;
        }}

        headerbar.obsidian-header button.titlebutton.close {{
            background: #FF5F56;
        }}

        headerbar.obsidian-header button.titlebutton.minimize {{
            background: #FFBD2E;
        }}

        headerbar.obsidian-header button.titlebutton.maximize {{
            background: #27C93F;
        }}
        
        headerbar.obsidian-header button.titlebutton.close:hover {{
            background: #FF3B30;
        }}
        
        headerbar.obsidian-header button.titlebutton.minimize:hover {{
            background: #E5A323;
        }}
        
        headerbar.obsidian-header button.titlebutton.maximize:hover {{
            background: #1CAD30;
        }}

        button.obsidian-header-settings {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 50%;
            min-height: 24px;
            min-width: 24px;
            padding: 4px;
            box-shadow: none;
            opacity: 0.4;
            transition: opacity 140ms ease;
        }}

        button.obsidian-header-settings:hover {{
            opacity: 1.0;
        }}

        .obsidian-logo {{
            opacity: 0.92;
        }}

        .obsidian-shell {{
            background: {window_bg};
            border-bottom-left-radius: 12px;
            border-bottom-right-radius: 12px;
        }}

        .obsidian-title {{
            color: {text_primary};
            font-weight: 700;
            letter-spacing: 0.04em;
        }}

        terminal.obsidian-terminal {{
            background: transparent;
            color: {text_primary};
            border: 1px solid {border};
            border-radius: 18px;
            padding: 10px;
        }}

        box.obsidian-workspace-actions {{
            background: transparent;
            border: none;
            margin: 0;
            padding: 0;
        }}

        separator.obsidian-separator {{
            background: {border};
            min-height: 1px;
            margin: 0 0 12px 0;
        }}

        separator.obsidian-v-separator {{
            background: {border};
            min-width: 1px;
            margin: 0 4px;
        }}

        .obsidian-right-pane {{
            background: transparent;
            padding: 20px;
            border: 1px solid {border};
            border-radius: 8px;
            margin: 0;
        }}

        .obsidian-handle {{
            min-width: 12px;
            margin: 0 2px;
            padding: 0;
            border-radius: 4px;
            transition: background 140ms ease;
        }}

        .obsidian-handle:hover {{
            background: rgba(255, 255, 255, 0.04);
        }}

        .obsidian-handle-dot {{
            color: {text_primary};
            font-size: 22px;
            opacity: 0.15;
            transition: opacity 140ms ease, color 140ms ease;
        }}

        .obsidian-handle:hover .obsidian-handle-dot {{
            opacity: 0.6;
            color: {accent};
        }}

        .obsidian-handle.collapsed .obsidian-handle-dot {{
            opacity: 0.4;
            color: {accent};
        }}

        box.obsidian-input-pill {{
            background: transparent;
            border: 1px solid {border};
            border-radius: 999px;
            padding: 4px 16px;
            margin: 0 0 8px 0;
            transition: border-color 140ms ease;
        }}

        box.obsidian-input-pill:focus-within {{
            border-color: {accent};
        }}

        box.obsidian-input-pill.terminal-active {{
            border-color: {accent};
        }}

        button.obsidian-workspace-button,
        button.obsidian-tool-button {{
            background: transparent;
            color: {text_primary};
            border-radius: 50%;
            padding: 8px;
            min-height: 28px;
            min-width: 28px;
            box-shadow: none;
            border: none;
            opacity: 0.6;
            transition: opacity 140ms ease, background 140ms ease;
        }}

        button.obsidian-search-toggle {{
            background: transparent;
            color: {text_primary};
            border-radius: 50%;
            padding: 2px;
            min-height: 18px;
            min-width: 18px;
            box-shadow: none;
            border: none;
            opacity: 0.35;
            transition: opacity 140ms ease;
        }}

        button.obsidian-search-toggle:hover {{
            opacity: 0.8;
        }}

        button.obsidian-workspace-button:hover,
        button.obsidian-tool-button:hover {{
            background: {surface};
            opacity: 1.0;
        }}

        .obsidian-tab-bar-container {{
            background: transparent;
            margin: 0 12px;
            padding: 4px 0;
            min-height: 40px;
        }}

        .obsidian-tab-bar-scroller {{
            background: transparent;
            margin-right: 8px;
            min-height: 40px;
        }}

        .obsidian-tabs-list {{
            background: transparent;
            padding: 0 4px;
        }}

        .obsidian-tab-item {{
            background: transparent;
            border-radius: 0;
            padding: 6px 14px;
            margin: 0 2px;
            transition: background 140ms ease, opacity 140ms ease, border-color 140ms ease;
            border-bottom: 2px solid transparent;
            opacity: 0.4;
        }}

        .obsidian-tab-item:hover {{
            background: rgba(255, 255, 255, 0.03);
            opacity: 0.8;
        }}

        .obsidian-tab-item.active {{
            background: transparent;
            border-bottom-color: {accent};
            opacity: 1.0;
        }}

        button.obsidian-tab-close-button {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 50%;
            min-height: 20px;
            min-width: 20px;
            padding: 0;
            margin-left: 2px;
            opacity: 0;
            box-shadow: none;
            transition: opacity 140ms ease, background 140ms ease;
        }}

        .obsidian-tab-item:hover button.obsidian-tab-close-button,
        .obsidian-tab-item.active button.obsidian-tab-close-button {{
            opacity: 0.4;
        }}

        button.obsidian-tab-close-button:hover {{
            background: rgba(255, 255, 255, 0.1);
            opacity: 1.0;
        }}

        .obsidian-tab-item.dragging {{
            opacity: 0.4;
        }}

        .obsidian-tab-item.drop-target {{
            background: rgba(255, 255, 255, 0.05);
            border-bottom-color: {accent};
        }}

        button.obsidian-add-tab-button {{
            background: transparent;
            color: {text_primary};
            border: none;
            padding: 6px;
            min-height: 28px;
            min-width: 28px;
            opacity: 0.5;
            margin: 0 8px;
            border-radius: 50%;
            transition: opacity 140ms ease, background 140ms ease;
        }}

        button.obsidian-add-tab-button:hover {{
            opacity: 1.0;
            background: rgba(255, 255, 255, 0.05);
        }}

        notebook.obsidian-notebook > stack {{
            background: transparent;
        }}

        label.obsidian-tab-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            font-weight: 700;
            text-transform: lowercase;
        }}

        entry.obsidian-entry.search-active {{
            color: #FFBD2E;
        }}

        box.obsidian-input-pill.search-active {{
            border-color: rgba(255, 189, 46, 0.3);
        }}

        label.obsidian-user-label {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            font-weight: 700;
        }}

        label.obsidian-path-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            font-weight: 700;
        }}

        label.obsidian-status-label {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            font-weight: 700;
            margin: 0 6px 0 0;
        }}

        label.obsidian-status-ok {{
            color: #7FB685;
        }}

        label.obsidian-status-error {{
            color: {accent};
        }}

        label.obsidian-status-running {{
            color: #E5C07B;
        }}

        entry.obsidian-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            padding: 8px 0;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            box-shadow: none;
            outline: none;
        }}

        entry.obsidian-entry:focus {{
            box-shadow: none;
            outline: none;
        }}

        /* Logr Pane */
        .obsidian-logr-root {{
            padding: 6px 8px;
        }}

        .obsidian-logr-header {{
            padding: 4px 0;
            margin-bottom: 4px;
        }}

        .obsidian-logr-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            font-weight: 700;
            opacity: 0.5;
            text-transform: uppercase;
            letter-spacing: 0.08em;
        }}

        .obsidian-logr-count {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            font-weight: 700;
            opacity: 0.7;
        }}

        .obsidian-logr-picker {{
            margin-bottom: 4px;
        }}

        menubutton.obsidian-logr-select > button {{
            background: transparent;
            border: 1px solid {border};
            border-radius: 6px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            color: {text_primary};
            min-height: 24px;
            padding: 2px 8px;
            box-shadow: none;
            opacity: 0.6;
            transition: opacity 140ms ease, border-color 140ms ease;
        }}

        menubutton.obsidian-logr-select > button:hover {{
            opacity: 1.0;
            border-color: {accent};
        }}

        popover.obsidian-logr-popover {{
            background: {window_bg};
            border: 1px solid {border};
            border-radius: 6px;
            padding: 4px 0;
        }}

        popover.obsidian-logr-popover > contents {{
            background: {window_bg};
            border-radius: 6px;
            padding: 0;
        }}

        .obsidian-logr-popover-list {{
            background: transparent;
        }}

        .obsidian-logr-popover-row {{
            background: transparent;
            padding: 0;
            border-radius: 4px;
            margin: 1px 4px;
            transition: background 100ms ease;
        }}

        .obsidian-logr-popover-row:hover {{
            background: rgba(255, 255, 255, 0.05);
        }}

        row.obsidian-logr-popover-row:focus {{
            background: rgba(255, 77, 77, 0.10);
        }}

        .obsidian-logr-popover-item {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            padding: 5px 8px;
            opacity: 0.7;
        }}

        .obsidian-logr-popover-row:hover .obsidian-logr-popover-item {{
            opacity: 1.0;
        }}

        button.obsidian-logr-icon-btn {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 50%;
            min-height: 22px;
            min-width: 22px;
            padding: 2px;
            opacity: 0.4;
            box-shadow: none;
            transition: opacity 140ms ease;
        }}

        button.obsidian-logr-icon-btn:hover {{
            opacity: 0.9;
        }}

        .obsidian-logr-controls {{
            padding: 4px 0;
            margin-bottom: 2px;
        }}

        .obsidian-logr-stream-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 9px;
            opacity: 0.4;
        }}

        entry.obsidian-logr-filter {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-bottom: 1px solid {border};
            border-radius: 0;
            padding: 4px 2px;
            margin-bottom: 4px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            box-shadow: none;
            outline: none;
            opacity: 0.6;
            transition: opacity 140ms ease, border-color 140ms ease;
        }}

        entry.obsidian-logr-filter:focus {{
            opacity: 1.0;
            border-color: {accent};
            box-shadow: none;
            outline: none;
        }}

        .obsidian-logr-status {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 9px;
            opacity: 0.35;
            padding: 4px 0;
        }}

        .obsidian-logr-empty {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            opacity: 0.3;
            padding: 12px 4px;
        }}

        .obsidian-log-list {{
            background: transparent;
        }}

        .obsidian-log-entry {{
            padding: 3px 4px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.02);
            transition: background 100ms ease;
        }}

        .obsidian-log-entry:hover {{
            background: rgba(255, 255, 255, 0.03);
        }}

        .log-level-dot {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 9px;
            font-weight: 700;
            min-width: 14px;
            min-height: 14px;
            border-radius: 3px;
            padding: 0;
        }}

        .log-body {{
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            color: {text_primary};
            opacity: 0.7;
        }}

        /* Log Level Colors */
        .log-error .log-level-dot {{ color: #FF5F56; background: rgba(255, 95, 86, 0.12); }}
        .log-error .log-body {{ color: #FF8079; opacity: 0.9; }}

        .log-warn .log-level-dot {{ color: #FFBD2E; background: rgba(255, 189, 46, 0.12); }}
        .log-warn .log-body {{ color: #FFD272; opacity: 0.85; }}

        .log-info .log-level-dot {{ color: #27C93F; background: rgba(39, 201, 63, 0.12); }}
        .log-info .log-body {{ color: #B0E4B7; opacity: 0.8; }}

        .log-debug .log-level-dot {{ color: {text_primary}; opacity: 0.3; background: rgba(255, 255, 255, 0.04); }}
        .log-debug .log-body {{ opacity: 0.4; }}

        /* Settings Page */
        .obsidian-settings-root {{
            padding: 16px 0 16px 24px;
        }}

        .obsidian-settings-header {{
            padding: 4px 24px 14px 0;
            margin-bottom: 4px;
        }}

        button.obsidian-settings-back {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-radius: 4px;
            min-height: 28px;
            min-width: 28px;
            padding: 4px;
            box-shadow: none;
            opacity: 0.4;
            transition: opacity 140ms ease;
        }}

        button.obsidian-settings-back:hover {{
            opacity: 1.0;
        }}

        .obsidian-settings-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 13px;
            font-weight: 700;
            text-transform: lowercase;
            letter-spacing: 0.06em;
            opacity: 0.85;
        }}

        .obsidian-settings-content {{
            padding: 0 18px 0 4px;
        }}

        .obsidian-settings-section {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.06em;
            padding: 20px 0 6px 0;
            margin-top: 2px;
            border-bottom: 1px solid rgba(255, 77, 77, 0.12);
        }}

        .obsidian-settings-row {{
            padding: 10px 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.025);
        }}

        .obsidian-settings-about {{
            padding: 12px 0;
        }}

        .obsidian-settings-label {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            opacity: 0.6;
        }}

        .obsidian-settings-value {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            opacity: 0.35;
        }}

        .obsidian-settings-about-copy {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            line-height: 1.5;
            opacity: 0.5;
            max-width: 72ch;
        }}

        entry.obsidian-settings-entry {{
            background: transparent;
            color: {text_primary};
            border: none;
            border-bottom: 1px solid {border};
            border-radius: 0;
            padding: 4px 6px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            box-shadow: none;
            outline: none;
            min-width: 160px;
            transition: border-color 140ms ease;
        }}

        entry.obsidian-settings-entry:focus {{
            border-color: {accent};
            box-shadow: none;
            outline: none;
        }}

        /* SpinButton: strip grey backgrounds */
        spinbutton.obsidian-settings-spin {{
            background: transparent;
            color: {text_primary};
            border: 1px solid {border};
            border-radius: 4px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            min-width: 80px;
            box-shadow: none;
        }}

        spinbutton.obsidian-settings-spin > text {{
            background: transparent;
        }}

        spinbutton.obsidian-settings-spin > button {{
            background: transparent;
            color: {text_primary};
            border: none;
            box-shadow: none;
            opacity: 0.4;
            transition: opacity 140ms ease;
        }}

        spinbutton.obsidian-settings-spin > button:hover {{
            opacity: 1.0;
            background: transparent;
        }}

        /* DropDown: strip grey backgrounds */
        .obsidian-settings-dropdown {{
            background: transparent;
            border: 1px solid {border};
            border-radius: 4px;
            box-shadow: none;
            min-width: 140px;
        }}

        .obsidian-settings-dropdown > button {{
            background: transparent;
            color: {text_primary};
            border: none;
            box-shadow: none;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            padding: 4px 8px;
            opacity: 0.7;
            transition: opacity 140ms ease;
        }}

        .obsidian-settings-dropdown > button:hover {{
            opacity: 1.0;
            background: transparent;
        }}

        .obsidian-settings-dropdown > popover {{
            background: {window_bg};
            border: 1px solid {border};
            border-radius: 6px;
        }}

        .obsidian-settings-dropdown > popover > contents {{
            background: {window_bg};
            border-radius: 6px;
            padding: 2px 0;
        }}

        .obsidian-settings-dropdown > popover listview {{
            background: transparent;
        }}

        .obsidian-settings-dropdown > popover listview > row {{
            background: transparent;
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            padding: 4px 8px;
            transition: background 100ms ease;
        }}

        .obsidian-settings-dropdown > popover listview > row:hover {{
            background: rgba(255, 255, 255, 0.04);
        }}

        .obsidian-settings-dropdown > popover listview > row:checked {{
            background: rgba(255, 77, 77, 0.10);
            color: {accent};
        }}

        /* Switch: strip grey backgrounds */
        switch.obsidian-settings-switch {{
            background: rgba(255, 255, 255, 0.06);
            border: 1px solid {border};
            border-radius: 999px;
            box-shadow: none;
            min-width: 42px;
            min-height: 22px;
            transition: background 140ms ease, border-color 140ms ease;
        }}

        switch.obsidian-settings-switch:checked {{
            background: rgba(255, 77, 77, 0.20);
            border-color: rgba(255, 77, 77, 0.35);
        }}

        switch.obsidian-settings-switch > image {{
            color: transparent;
        }}

        switch.obsidian-settings-switch slider {{
            background: rgba(255, 255, 255, 0.3);
            border: none;
            border-radius: 50%;
            box-shadow: none;
            min-width: 16px;
            min-height: 16px;
            margin: 2px;
            transition: background 140ms ease;
        }}

        switch.obsidian-settings-switch:checked slider {{
            background: {accent};
        }}

        .obsidian-settings-footer {{
            padding: 20px 0 8px 0;
        }}

        button.obsidian-settings-link {{
            background: transparent;
            color: {accent};
            border: 1px solid rgba(255, 77, 77, 0.18);
            border-radius: 999px;
            padding: 4px 12px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            box-shadow: none;
            transition: background 140ms ease, border-color 140ms ease;
        }}

        button.obsidian-settings-link:hover {{
            background: rgba(255, 77, 77, 0.08);
            border-color: rgba(255, 77, 77, 0.30);
        }}

        .obsidian-settings-about-page {{
            padding: 24px;
        }}

        .obsidian-settings-about-panel {{
            max-width: 640px;
        }}

        .obsidian-settings-about-title {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 22px;
            font-weight: 700;
            text-transform: lowercase;
            letter-spacing: 0.06em;
            opacity: 0.96;
        }}

        .obsidian-settings-about-name {{
            color: {accent};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 12px;
            font-weight: 700;
            letter-spacing: 0.08em;
            opacity: 0.9;
            margin-top: 8px;
        }}

        .obsidian-settings-about-meta {{
            color: {text_primary};
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 10px;
            opacity: 0.38;
            margin-top: 4px;
        }}

        button.obsidian-settings-save {{
            background: transparent;
            color: {accent};
            border: 1px solid {accent};
            border-radius: 4px;
            padding: 6px 24px;
            font-family: \"DejaVu Sans Mono\", monospace;
            font-size: 11px;
            font-weight: 700;
            box-shadow: none;
            transition: background 140ms ease;
        }}

        button.obsidian-settings-save:hover {{
            background: rgba(255, 77, 77, 0.10);
        }}
        ",
        window_bg = css_color(theme::BG_PRIMARY),
        window_edge = css_color(theme::WINDOW_EDGE),
        titlebar_bg = css_color(theme::BG_TITLEBAR),
        surface = css_color(theme::SURFACE_BASE),
        border = css_color(theme::BORDER_STRONG),
        text_primary = css_color(theme::TEXT_PRIMARY),
        accent = css_color(theme::ACCENT),
    );
    provider.load_from_data(&css);

    if let Some(display) = gdk::Display::default() {
        style_context_add_provider_for_display(
            &display,
            &provider,
            STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn css_color(color: u32) -> String {
    format!("#{:06X}", color & 0x00FF_FFFF)
}
