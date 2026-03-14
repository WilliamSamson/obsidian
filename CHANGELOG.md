# Changelog

## v0.1.0-beta.1 — 2026-03-14

First public beta release.

### Terminal Workspace
- GTK4/VTE terminal with tabs, split panes, and pane-local multiplexer sessions
- Tab rename, reorder, and quick switcher
- Keyboard session cycling and direct jump with `Ctrl+Alt+1..9`
- Clipboard integration and command history
- In-app command completion notices with desktop notification support
- Terminal inspector for debugging font, PTY, and rendering state
- Sixel image rendering and ligature shaping toggles
- Terminal selection copies on highlight

### Side Panes
- `logr` JSON log viewer with live follow, level filtering, search, and export
- Embedded web browser pane with selectable default search engine
- Collapsible side pane layout

### Settings
- First-run setup wizard with 3-step flow (runtime, workspace, appearance)
- Setup checkpoint persistence for crash recovery
- Autosaving settings with live preview
- Desktop notifications toggle
- Configurable shell command, font family, font sizes, cursor style, scrollback

### Workspace Persistence
- Tab layout, split position, and active pane restored on relaunch
- Window size and position remembered across sessions

### Design
- Custom dark terminal-native visual system
- macOS-style window controls with hover icons and tooltips
- Flat, minimal interface with no gradients or box-shadows

### Packaging
- `.deb` package for Ubuntu 24.04+ with proper dependency declarations
- AppImage build support via AppDir bundling
- Per-user `install.sh` / `uninstall.sh` scripts
- Desktop entry and icon integration

### Security
- WebKit sandbox only disabled in bundled AppImage mode
- Process-scoped temporary files to prevent race conditions
- Error logging on settings and checkpoint save failures
- No unsafe code in the codebase

### Log Viewer (standalone mode)
- File-based and piped stdin log ingestion
- JSONL parsing with graceful malformed line handling
- Level filtering, text search, and filtered export
- Startup filters via `--filter` arguments

### License
- Licensed under GNU General Public License v3.0
