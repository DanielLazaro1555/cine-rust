# Graph Report - cine-rust  (2026-10-29)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 262 nodes · 472 edges · 26 communities (12 shown, 14 thin omitted)
- Extraction: 93% EXTRACTED · 7% INFERRED · 0% AMBIGUOUS · INFERRED: 32 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c3270da6`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- CineWindow
- mpv_engine.rs
- VideoGLArea
- create_options_menu_button
- present_playlist_dialog
- utils.rs
- .on_startup
- history.rs
- AppSettings
- MprisController
- main.rs
- Cine (Rust Edition)
- install.sh
- uninstall.sh
- ExitCode
- cine-rust
- c_void
- Option
- PathBuf
- String
- Box
- Button
- MenuButton
- Rc
- RefCell
- Vec

## God Nodes (most connected - your core abstractions)
1. `CineWindow` - 40 edges
2. `VideoGLArea` - 38 edges
3. `MpvEngine` - 16 edges
4. `MpvHandle` - 11 edges
5. `create_options_menu_button()` - 9 edges
6. `MprisController` - 9 edges
7. `save_playlist_session()` - 8 edges
8. `present_playlist_dialog()` - 8 edges
9. `mpv_render_context_set_update_callback()` - 7 edges
10. `make_option_row()` - 7 edges

## Surprising Connections (you probably didn't know these)
- `record_history_entry()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs
- `last_playlist_file()` --calls--> `config_dir()`  [INFERRED]
  src/save_session.rs → src/utils.rs
- `clear_history()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs
- `read_history_entries()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs
- `save_playlist_session()` --references--> `PlaylistItem`  [EXTRACTED]
  src/save_session.rs → src/playlist.rs

## Import Cycles
- None detected.

## Communities (26 total, 14 thin omitted)

### Community 0 - "CineWindow"
Cohesion: 0.09
Nodes (26): Adjustment, ApplicationWindow, Box, Button, File, Label, MenuButton, MprisController (+18 more)

### Community 1 - "mpv_engine.rs"
Cohesion: 0.11
Nodes (36): c_char, c_int, c_uint, Drop, MpvRenderUpdateFn, eglGetProcAddress(), get_proc_address_trampoline(), GL_FRAMEBUFFER_BINDING (+28 more)

### Community 2 - "VideoGLArea"
Cohesion: 0.08
Nodes (12): c_void, Cell, GLArea, GraphicsOffload, MpvEngine, on_mpv_render_update(), Option, Rc (+4 more)

### Community 3 - "create_options_menu_button"
Cohesion: 0.19
Nodes (14): SpinButton, create_options_menu_button(), make_option_row(), make_spin_row(), PlaybackOptions, Box, Button, Default (+6 more)

### Community 4 - "present_playlist_dialog"
Cohesion: 0.18
Nodes (13): FClear, FRemove, FSelect, PlaylistItem, present_playlist_dialog(), IsA, PathBuf, String (+5 more)

### Community 5 - "utils.rs"
Cohesion: 0.22
Nodes (12): Path, PathBuf, calculate_window_size(), collect_media_files(), config_dir(), format_time(), is_media_file(), resolution_badge() (+4 more)

### Community 6 - ".on_startup"
Cohesion: 0.17
Nodes (10): CineApplication, Application, ExitCode, Self, SharedSettings, present_shortcuts_dialog(), register_window_accels(), Application (+2 more)

### Community 7 - "history.rs"
Cohesion: 0.24
Nodes (11): FOpen, clear_history(), HistoryEntry, parse_json_line(), present_history_dialog(), read_history_entries(), IsA, Option (+3 more)

### Community 8 - "AppSettings"
Cohesion: 0.21
Nodes (8): AppSettings, present_preferences_dialog(), Default, IsA, Self, SharedSettings, String, Widget

### Community 9 - "MprisController"
Cohesion: 0.25
Nodes (6): MprisController, Default, Rc, RefCell, Self, String

### Community 10 - "main.rs"
Cohesion: 0.29
Nodes (6): APP_ID, GRESOURCE_BYTES, ICON_NAME, main(), ExitCode, VERSION

### Community 11 - "Cine (Rust Edition)"
Cohesion: 0.40
Nodes (5): Cine (Rust Edition), Daniel Lazaro, Diego Povliuk, GtkGraphicsOffload, Aceleración HWDEC / NVDEC

## Knowledge Gaps
- **12 isolated node(s):** `APP_ID`, `GRESOURCE_BYTES`, `ICON_NAME`, `VERSION`, `install.sh script` (+7 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 88 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **14 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `MpvEngine` connect `mpv_engine.rs` to `CineWindow`?**
  _High betweenness centrality (0.222) - this node is a cross-community bridge._
- **Why does `VideoGLArea` connect `VideoGLArea` to `create_options_menu_button`?**
  _High betweenness centrality (0.214) - this node is a cross-community bridge._
- **Why does `CineWindow` connect `CineWindow` to `VideoGLArea`, `utils.rs`?**
  _High betweenness centrality (0.171) - this node is a cross-community bridge._
- **What connects `APP_ID`, `GRESOURCE_BYTES`, `ICON_NAME` to the rest of the system?**
  _12 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `CineWindow` be split into smaller, more focused modules?**
  _Cohesion score 0.09219858156028368 - nodes in this community are weakly interconnected._
- **Should `mpv_engine.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1099290780141844 - nodes in this community are weakly interconnected._
- **Should `VideoGLArea` be split into smaller, more focused modules?**
  _Cohesion score 0.07781649245063879 - nodes in this community are weakly interconnected._