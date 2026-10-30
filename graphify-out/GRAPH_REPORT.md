# Graph Report - cine-rust  (2026-10-30)

## Corpus Check
- 28 files · ~20,312 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 56 file(s) not represented in the graph (top: .po 26, .blp 6, .ui 6)

## Summary
- 329 nodes · 544 edges · 27 communities (16 shown, 5 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 35 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `30b37859`
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
- history.rs
- utils.rs
- .open_files
- main.rs
- Cine (Rust Edition)
- install.sh
- uninstall.sh
- Q: ¿Cómo conecta VideoGLArea el motor mpv con la ventana y los controles del reproductor?
- cine-rust
- setup_video_gestures
- CineWindow
- MprisController
- build_headerbar
- .setup_mouse_gestures

## God Nodes (most connected - your core abstractions)
1. `VideoGLArea` - 49 edges
2. `CineWindow` - 24 edges
3. `MpvEngine` - 18 edges
4. `MprisController` - 12 edges
5. `build_playback_controls()` - 12 edges
6. `MpvHandle` - 11 edges
7. `PlaybackControlsUi` - 11 edges
8. `present_playlist_dialog()` - 10 edges
9. `setup_video_gestures()` - 9 edges
10. `create_options_menu_button()` - 9 edges

## Surprising Connections (you probably didn't know these)
- `record_history_entry()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs
- `read_history_entries()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs
- `clear_history()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs
- `build_playback_controls()` --calls--> `create_options_menu_button()`  [INFERRED]
  src/window/ui.rs → src/options.rs
- `last_playlist_file()` --calls--> `config_dir()`  [INFERRED]
  src/save_session.rs → src/utils.rs

## Import Cycles
- None detected.

## Communities (27 total, 5 thin omitted)

### Community 0 - "CineWindow"
Cohesion: 0.12
Nodes (18): SourceId, CineWindow, Adjustment, ApplicationWindow, Box, Button, Cell, HeaderBar (+10 more)

### Community 1 - "mpv_engine.rs"
Cohesion: 0.11
Nodes (36): c_char, c_int, c_uint, Drop, MpvRenderUpdateFn, eglGetProcAddress(), get_proc_address_trampoline(), GL_FRAMEBUFFER_BINDING (+28 more)

### Community 2 - "VideoGLArea"
Cohesion: 0.08
Nodes (10): GLArea, GraphicsOffload, on_mpv_render_update(), c_void, Option, Rc, RefCell, Self (+2 more)

### Community 3 - "create_options_menu_button"
Cohesion: 0.19
Nodes (14): SpinButton, create_options_menu_button(), make_option_row(), make_spin_row(), PlaybackOptions, Box, Button, Default (+6 more)

### Community 4 - "present_playlist_dialog"
Cohesion: 0.13
Nodes (17): FAddFiles, FAddUrl, FClear, FRemove, FSelect, PlaylistItem, present_playlist_dialog(), IsA (+9 more)

### Community 5 - "build_playback_controls"
Cohesion: 0.11
Nodes (25): FAdd, Scale, create_audio_menu_button(), create_subtitles_menu_button(), format_track_title(), get_available_tracks(), lang_code_to_name(), MenuButton (+17 more)

### Community 6 - ".on_startup"
Cohesion: 0.09
Nodes (18): CineApplication, Application, ExitCode, Self, SharedSettings, AppSettings, present_preferences_dialog(), Default (+10 more)

### Community 7 - "history.rs"
Cohesion: 0.24
Nodes (11): FOpen, clear_history(), HistoryEntry, parse_json_line(), present_history_dialog(), read_history_entries(), IsA, Option (+3 more)

### Community 8 - "utils.rs"
Cohesion: 0.13
Nodes (16): Path, calculate_window_size(), center_active_cine_window(), collect_media_files(), config_dir(), format_time(), install_cine_window_added_centering(), is_media_file() (+8 more)

### Community 9 - ".open_files"
Cohesion: 0.23
Nodes (5): File, FToast, record_history_entry(), CineWindow, ApplicationWindow

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
Cohesion: 0.24
Nodes (9): F, L, M, ApplicationWindow, Cell, Overlay, Rc, StatusPage (+1 more)

### Community 23 - "MprisController"
Cohesion: 0.16
Nodes (10): MprisController, Default, Rc, RefCell, Self, String, CineWindow, Application (+2 more)

### Community 24 - "build_headerbar"
Cohesion: 0.33
Nodes (5): build_headerbar(), build_start_page(), HeaderBar, StatusPage, WindowTitle

## Knowledge Gaps
- **17 isolated node(s):** `cine-rust`, `install.sh script`, `APP_ID`, `ICON_NAME`, `VERSION` (+12 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 116 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `VideoGLArea` connect `VideoGLArea` to `CineWindow`, `mpv_engine.rs`, `create_options_menu_button`, `build_playback_controls`, `.open_files`?**
  _High betweenness centrality (0.376) - this node is a cross-community bridge._
- **Why does `MpvEngine` connect `mpv_engine.rs` to `VideoGLArea`?**
  _High betweenness centrality (0.218) - this node is a cross-community bridge._
- **Why does `CineWindow` connect `CineWindow` to `VideoGLArea`, `create_options_menu_button`, `present_playlist_dialog`, `utils.rs`, `MprisController`?**
  _High betweenness centrality (0.144) - this node is a cross-community bridge._
- **What connects `cine-rust`, `install.sh script`, `APP_ID` to the rest of the system?**
  _17 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `CineWindow` be split into smaller, more focused modules?**
  _Cohesion score 0.11578947368421053 - nodes in this community are weakly interconnected._
- **Should `mpv_engine.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.10714285714285714 - nodes in this community are weakly interconnected._
- **Should `VideoGLArea` be split into smaller, more focused modules?**
  _Cohesion score 0.08194905869324474 - nodes in this community are weakly interconnected._