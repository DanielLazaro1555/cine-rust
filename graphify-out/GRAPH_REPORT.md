# Graph Report - cine-rust  (2026-10-31)

## Corpus Check
- 25 files · ~24,715 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 409 nodes · 686 edges · 32 communities (20 shown, 7 thin omitted)
- Extraction: 96% EXTRACTED · 4% INFERRED · 0% AMBIGUOUS · INFERRED: 30 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- GTK Video Rendering
- libmpv and OpenGL
- Playlist Management
- Display and File Utilities
- Audio and Subtitle Tracks
- Persistent Playback Logs
- Application Startup
- Playback Preferences
- Rendering and Sync Documentation
- Playback Actions
- Playback History
- Gesture Input
- MPRIS Integration
- Keyboard Shortcuts
- Window Controls
- Executable Entry Point
- Window Initialization
- Window Surface UI
- Playback UI and Engine
- Architecture Q&A
- Zoomed Video Panning
- Render Diagnostics
- Installer
- Uninstaller
- Thread Synchronization
- Project Overview
- Process Exit Status

## God Nodes (most connected - your core abstractions)
1. `VideoGLArea` - 55 edges
2. `CineWindow` - 26 edges
3. `MpvEngine` - 18 edges
4. `build_playback_controls()` - 16 edges
5. `create_options_menu_button()` - 12 edges
6. `x11_surface_position()` - 12 edges
7. `setup_video_gestures()` - 11 edges
8. `MpvHandle` - 11 edges
9. `center_x11_surface()` - 11 edges
10. `move_x11_surface()` - 11 edges

## Surprising Connections (you probably didn't know these)
- `GTK frame clock` --conceptually_related_to--> `GtkGLArea`  [EXTRACTED]
  docs/reproduccion-y-renderizado.md → README.md
- `libmpv` --conceptually_related_to--> `GtkGLArea`  [EXTRACTED]
  docs/reproduccion-y-renderizado.md → README.md
- `last_playlist_file()` --calls--> `config_dir()`  [INFERRED]
  src/save_session.rs → src/utils.rs
- `record_history_entry()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs
- `build_playback_controls()` --calls--> `create_options_menu_button()`  [INFERRED]
  src/window/ui.rs → src/options.rs

## Import Cycles
- None detected.

## Communities (32 total, 7 thin omitted)

### Community 0 - "GTK Video Rendering"
Cohesion: 0.06
Nodes (19): Arc, AtomicBool, Duration, GLArea, GraphicsOffload, Instant, MpvRenderCallbackContext, on_mpv_render_update() (+11 more)

### Community 1 - "libmpv and OpenGL"
Cohesion: 0.11
Nodes (36): c_char, c_uint, Drop, MpvRenderUpdateFn, eglGetProcAddress(), get_proc_address_trampoline(), GL_FRAMEBUFFER_BINDING, glGetIntegerv() (+28 more)

### Community 2 - "Playlist Management"
Cohesion: 0.07
Nodes (35): FAddFiles, FAddUrl, FClear, FRemove, FSelect, PlaylistItem, present_playlist_dialog(), IsA (+27 more)

### Community 3 - "Display and File Utilities"
Cohesion: 0.13
Nodes (31): c_ulong, Display, Path, center_cine_window(), CENTER_SCRIPT_LOCK, center_x11_surface(), collect_media_files(), config_dir() (+23 more)

### Community 4 - "Audio and Subtitle Tracks"
Cohesion: 0.11
Nodes (26): FAdd, Scale, create_audio_menu_button(), create_subtitles_menu_button(), format_track_title(), get_available_tracks(), lang_code_to_name(), MenuButton (+18 more)

### Community 5 - "Persistent Playback Logs"
Cohesion: 0.13
Nodes (12): File, FToast, Result, init(), log_path(), MAX_LOG_BYTES, redirect_output(), PathBuf (+4 more)

### Community 6 - "Application Startup"
Cohesion: 0.13
Nodes (12): CineApplication, Application, ExitCode, Self, SharedSettings, AppSettings, present_preferences_dialog(), Default (+4 more)

### Community 7 - "Playback Preferences"
Cohesion: 0.15
Nodes (17): SpinButton, create_options_menu_button(), make_option_row(), make_spin_row(), PlaybackOptions, Box, Button, Default (+9 more)

### Community 8 - "Rendering and Sync Documentation"
Cohesion: 0.11
Nodes (18): Display synchronization, GTK frame clock, libmpv, Main-thread render wait, mpv reference manual, libmpv render API, video-timing-offset, Cine Rust (+10 more)

### Community 9 - "Playback Actions"
Cohesion: 0.21
Nodes (5): calculate_window_size(), format_time(), resolution_badge(), String, CineWindow

### Community 10 - "Playback History"
Cohesion: 0.22
Nodes (13): FOpen, clear_history(), HistoryEntry, parse_json_line(), present_history_dialog(), read_history_entries(), record_history_entry(), IsA (+5 more)

### Community 11 - "Gesture Input"
Cohesion: 0.20
Nodes (11): D, F, L, M, ApplicationWindow, Cell, Overlay, Rc (+3 more)

### Community 12 - "MPRIS Integration"
Cohesion: 0.25
Nodes (6): MprisController, Default, Rc, RefCell, Self, String

### Community 13 - "Keyboard Shortcuts"
Cohesion: 0.20
Nodes (7): present_shortcuts_dialog(), register_window_accels(), Application, IsA, Widget, SHORTCUTS_UI_XML, CineWindow

### Community 15 - "Executable Entry Point"
Cohesion: 0.29
Nodes (6): ExitCode, APP_ID, GRESOURCE_BYTES, ICON_NAME, main(), VERSION

### Community 16 - "Window Initialization"
Cohesion: 0.33
Nodes (5): CineWindow, Application, MprisController, Self, SharedSettings

### Community 17 - "Window Surface UI"
Cohesion: 0.33
Nodes (5): build_headerbar(), build_start_page(), HeaderBar, StatusPage, WindowTitle

### Community 18 - "Playback UI and Engine"
Cohesion: 0.50
Nodes (5): CineWindow, GTK render callback, MpvEngine, Playback controls, VideoGLArea

### Community 19 - "Architecture Q&A"
Cohesion: 0.40
Nodes (4): Answer, Outcome, Q: ¿Cómo conecta VideoGLArea el motor mpv con la ventana y los controles del reproductor?, Source Nodes

### Community 20 - "Zoomed Video Panning"
Cohesion: 0.67
Nodes (3): Frame-clock pan coalescing, Synchronous property reads during drag, Zoomed video panning

## Knowledge Gaps
- **27 isolated node(s):** `uninstall.sh script`, `Answer`, `Outcome`, `Source Nodes`, `cine-rust` (+22 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 135 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **7 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `VideoGLArea` connect `GTK Video Rendering` to `libmpv and OpenGL`, `Playlist Management`, `Audio and Subtitle Tracks`, `Persistent Playback Logs`, `Playback Preferences`?**
  _High betweenness centrality (0.323) - this node is a cross-community bridge._
- **Why does `MpvEngine` connect `libmpv and OpenGL` to `GTK Video Rendering`, `Persistent Playback Logs`?**
  _High betweenness centrality (0.184) - this node is a cross-community bridge._
- **Why does `CineWindow` connect `Playlist Management` to `GTK Video Rendering`, `Display and File Utilities`, `Playback Preferences`?**
  _High betweenness centrality (0.098) - this node is a cross-community bridge._
- **Are the 4 inferred relationships involving `build_playback_controls()` (e.g. with `create_options_menu_button()` and `create_audio_menu_button()`) actually correct?**
  _`build_playback_controls()` has 4 INFERRED edges - model-reasoned connections that need verification._
- **What connects `uninstall.sh script`, `Answer`, `Outcome` to the rest of the system?**
  _27 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `GTK Video Rendering` be split into smaller, more focused modules?**
  _Cohesion score 0.06453634085213032 - nodes in this community are weakly interconnected._
- **Should `libmpv and OpenGL` be split into smaller, more focused modules?**
  _Cohesion score 0.1099290780141844 - nodes in this community are weakly interconnected._