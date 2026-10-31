# Graph Report - cine-rust  (2026-10-31)

## Corpus Check
- 28 files · ~21,792 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 56 file(s) not represented in the graph (top: .po 26, .blp 6, .ui 6)

## Summary
- 363 nodes · 640 edges · 25 communities (16 shown, 4 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 38 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `e5dadfea`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- CineWindow
- mpv_engine.rs
- VideoGLArea
- create_options_menu_button
- present_playlist_dialog
- build_playback_controls
- .on_startup
- .open_files
- CineWindow
- main.rs
- Cine (Rust Edition)
- install.sh
- uninstall.sh
- Q: ¿Cómo conecta VideoGLArea el motor mpv con la ventana y los controles del reproductor?
- cine-rust
- setup_video_gestures
- CineWindow
- utils.rs
- MprisController
- build_headerbar

## God Nodes (most connected - your core abstractions)
1. `VideoGLArea` - 53 edges
2. `CineWindow` - 26 edges
3. `MpvEngine` - 18 edges
4. `build_playback_controls()` - 16 edges
5. `MprisController` - 12 edges
6. `create_options_menu_button()` - 12 edges
7. `x11_surface_position()` - 12 edges
8. `MpvHandle` - 11 edges
9. `center_x11_surface()` - 11 edges
10. `move_x11_surface()` - 11 edges

## Surprising Connections (you probably didn't know these)
- `build_playback_controls()` --calls--> `create_options_menu_button()`  [INFERRED]
  src/window/ui.rs → src/options.rs
- `last_playlist_file()` --calls--> `config_dir()`  [INFERRED]
  src/save_session.rs → src/utils.rs
- `build_playback_controls()` --calls--> `format_time()`  [INFERRED]
  src/window/ui.rs → src/utils.rs
- `record_history_entry()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs
- `read_history_entries()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs

## Import Cycles
- None detected.

## Communities (25 total, 4 thin omitted)

### Community 0 - "CineWindow"
Cohesion: 0.14
Nodes (18): CineWindow, Adjustment, ApplicationWindow, Box, Button, Cell, HeaderBar, Label (+10 more)

### Community 1 - "mpv_engine.rs"
Cohesion: 0.11
Nodes (36): c_char, c_uint, Drop, MpvRenderUpdateFn, eglGetProcAddress(), get_proc_address_trampoline(), GL_FRAMEBUFFER_BINDING, glGetIntegerv() (+28 more)

### Community 2 - "VideoGLArea"
Cohesion: 0.08
Nodes (10): GLArea, GraphicsOffload, on_mpv_render_update(), c_void, Option, Rc, RefCell, Self (+2 more)

### Community 3 - "create_options_menu_button"
Cohesion: 0.15
Nodes (17): SpinButton, create_options_menu_button(), make_option_row(), make_spin_row(), PlaybackOptions, Box, Button, Default (+9 more)

### Community 4 - "present_playlist_dialog"
Cohesion: 0.11
Nodes (20): FAddFiles, FAddUrl, FClear, FRemove, FSelect, PlaylistItem, present_playlist_dialog(), IsA (+12 more)

### Community 5 - "build_playback_controls"
Cohesion: 0.11
Nodes (26): FAdd, Scale, create_audio_menu_button(), create_subtitles_menu_button(), format_track_title(), get_available_tracks(), lang_code_to_name(), MenuButton (+18 more)

### Community 6 - ".on_startup"
Cohesion: 0.11
Nodes (15): CineApplication, Application, ExitCode, Self, SharedSettings, AppSettings, present_preferences_dialog(), Default (+7 more)

### Community 7 - ".open_files"
Cohesion: 0.10
Nodes (19): File, FOpen, FToast, clear_history(), HistoryEntry, parse_json_line(), present_history_dialog(), read_history_entries() (+11 more)

### Community 8 - "CineWindow"
Cohesion: 0.21
Nodes (5): calculate_window_size(), format_time(), resolution_badge(), String, CineWindow

### Community 10 - "main.rs"
Cohesion: 0.29
Nodes (6): APP_ID, GRESOURCE_BYTES, ICON_NAME, main(), ExitCode, VERSION

### Community 11 - "Cine (Rust Edition)"
Cohesion: 0.40
Nodes (5): Cine (Rust Edition), Daniel Lazaro, Diego Povliuk, GtkGraphicsOffload, Aceleración HWDEC / NVDEC

### Community 14 - "Q: ¿Cómo conecta VideoGLArea el motor mpv con la ventana y los controles del reproductor?"
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: ¿Cómo conecta VideoGLArea el motor mpv con la ventana y los controles del reproductor?, Source Nodes

### Community 16 - "setup_video_gestures"
Cohesion: 0.22
Nodes (10): D, F, L, M, ApplicationWindow, Cell, Overlay, Rc (+2 more)

### Community 18 - "utils.rs"
Cohesion: 0.13
Nodes (31): c_ulong, Display, Mutex, Path, center_cine_window(), CENTER_SCRIPT_LOCK, center_x11_surface(), collect_media_files() (+23 more)

### Community 23 - "MprisController"
Cohesion: 0.16
Nodes (10): MprisController, Default, Rc, RefCell, Self, String, CineWindow, Application (+2 more)

### Community 24 - "build_headerbar"
Cohesion: 0.33
Nodes (5): build_headerbar(), build_start_page(), HeaderBar, StatusPage, WindowTitle

## Knowledge Gaps
- **17 isolated node(s):** `cine-rust`, `install.sh script`, `APP_ID`, `ICON_NAME`, `VERSION` (+12 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 114 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `VideoGLArea` connect `VideoGLArea` to `CineWindow`, `mpv_engine.rs`, `create_options_menu_button`, `build_playback_controls`, `.open_files`?**
  _High betweenness centrality (0.378) - this node is a cross-community bridge._
- **Why does `MpvEngine` connect `mpv_engine.rs` to `VideoGLArea`, `.open_files`?**
  _High betweenness centrality (0.205) - this node is a cross-community bridge._
- **Why does `CineWindow` connect `CineWindow` to `VideoGLArea`, `create_options_menu_button`, `present_playlist_dialog`, `utils.rs`, `MprisController`?**
  _High betweenness centrality (0.136) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `build_playback_controls()` (e.g. with `create_options_menu_button()` and `create_audio_menu_button()`) actually correct?**
  _`build_playback_controls()` has 4 INFERRED edges - model-reasoned connections that need verification._
- **What connects `cine-rust`, `install.sh script`, `APP_ID` to the rest of the system?**
  _17 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `CineWindow` be split into smaller, more focused modules?**
  _Cohesion score 0.1380952380952381 - nodes in this community are weakly interconnected._
- **Should `mpv_engine.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1099290780141844 - nodes in this community are weakly interconnected._