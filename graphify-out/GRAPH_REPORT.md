# Graph Report - cine-rust  (2026-10-29)

## Corpus Check
- Corpus is ~13,995 words - fits in a single context window. You may not need a graph.

## Summary
- 192 nodes · 346 edges · 11 communities (10 shown, 1 thin omitted)
- Extraction: 95% EXTRACTED · 5% INFERRED · 0% AMBIGUOUS · INFERRED: 18 edges (avg confidence: 0.85)
- Token cost: 250 input · 150 output

## Community Hubs (Navigation)
- MpvEngine FFI & Hardware Decoding
- GTK4 UI & Window Management
- VideoGLArea OpenGL & Graphics Offload
- CineApplication Lifecycle & Global Actions
- Watch History & Session Persistence
- Application Entry Point & GResource
- Playlist Management & Dialogs
- MPRIS D-Bus Media Controller
- Project Architecture & Authorship
- Playback Options & Aspect Ratio
- Cargo Package Manifest

## God Nodes (most connected - your core abstractions)
1. `CineWindow` - 38 edges
2. `VideoGLArea` - 23 edges
3. `MpvEngine` - 18 edges
4. `MpvHandle` - 11 edges
5. `MprisController` - 8 edges
6. `mpv_render_context_set_update_callback()` - 7 edges
7. `CineApplication` - 6 edges
8. `MpvRenderContext` - 6 edges
9. `mpv_get_property_string()` - 6 edges
10. `mpv_render_context_create()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `present_history_dialog()` --calls--> `watch_history_path()`  [INFERRED]
  src/history.rs → src/utils.rs
- `CineWindow` --references--> `MprisController`  [EXTRACTED]
  src/window.rs → src/mpris.rs
- `VideoGLArea` --references--> `MpvEngine`  [EXTRACTED]
  src/mpv_gl_area.rs → src/mpv_engine.rs
- `CineWindow` --references--> `VideoGLArea`  [EXTRACTED]
  src/window.rs → src/mpv_gl_area.rs
- `CineWindow` --references--> `PlaybackOptions`  [EXTRACTED]
  src/window.rs → src/options.rs

## Import Cycles
- None detected.

## Communities (11 total, 1 thin omitted)

### Community 0 - "MpvEngine FFI & Hardware Decoding"
Cohesion: 0.11
Nodes (36): c_char, c_int, c_uint, Drop, MpvRenderUpdateFn, eglGetProcAddress(), get_proc_address_trampoline(), GL_FRAMEBUFFER_BINDING (+28 more)

### Community 1 - "GTK4 UI & Window Management"
Cohesion: 0.11
Nodes (18): Adjustment, ApplicationWindow, Box, Button, File, Label, MenuButton, Overlay (+10 more)

### Community 2 - "VideoGLArea OpenGL & Graphics Offload"
Cohesion: 0.10
Nodes (11): Cell, GLArea, GraphicsOffload, on_mpv_render_update(), c_void, Option, Rc, RefCell (+3 more)

### Community 3 - "CineApplication Lifecycle & Global Actions"
Cohesion: 0.09
Nodes (18): CineApplication, Application, ExitCode, Self, SharedSettings, AppSettings, present_preferences_dialog(), Default (+10 more)

### Community 4 - "Watch History & Session Persistence"
Cohesion: 0.12
Nodes (19): Path, present_history_dialog(), IsA, Widget, last_playlist_file(), PathBuf, calculate_window_size(), center_window_on_screen() (+11 more)

### Community 5 - "Application Entry Point & GResource"
Cohesion: 0.29
Nodes (6): APP_ID, GRESOURCE_BYTES, ICON_NAME, main(), ExitCode, VERSION

### Community 6 - "Playlist Management & Dialogs"
Cohesion: 0.38
Nodes (6): PlaylistItem, present_playlist_dialog(), IsA, PathBuf, String, Widget

### Community 7 - "MPRIS D-Bus Media Controller"
Cohesion: 0.33
Nodes (3): MprisController, Self, String

### Community 8 - "Project Architecture & Authorship"
Cohesion: 0.40
Nodes (5): Cine (Rust Edition), Daniel Lazaro, Diego Povliuk, GtkGraphicsOffload, Aceleración HWDEC / NVDEC

### Community 9 - "Playback Options & Aspect Ratio"
Cohesion: 0.40
Nodes (3): PlaybackOptions, Default, Self

## Knowledge Gaps
- **10 isolated node(s):** `cine-rust`, `APP_ID`, `ICON_NAME`, `VERSION`, `GRESOURCE_BYTES` (+5 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 69 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **1 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `VideoGLArea` connect `VideoGLArea OpenGL & Graphics Offload` to `MpvEngine FFI & Hardware Decoding`, `GTK4 UI & Window Management`?**
  _High betweenness centrality (0.419) - this node is a cross-community bridge._
- **Why does `CineWindow` connect `GTK4 UI & Window Management` to `VideoGLArea OpenGL & Graphics Offload`, `Watch History & Session Persistence`, `Playlist Management & Dialogs`, `MPRIS D-Bus Media Controller`, `Playback Options & Aspect Ratio`?**
  _High betweenness centrality (0.391) - this node is a cross-community bridge._
- **Why does `MpvEngine` connect `MpvEngine FFI & Hardware Decoding` to `GTK4 UI & Window Management`, `VideoGLArea OpenGL & Graphics Offload`?**
  _High betweenness centrality (0.337) - this node is a cross-community bridge._
- **What connects `cine-rust`, `APP_ID`, `ICON_NAME` to the rest of the system?**
  _10 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `MpvEngine FFI & Hardware Decoding` be split into smaller, more focused modules?**
  _Cohesion score 0.1099290780141844 - nodes in this community are weakly interconnected._
- **Should `GTK4 UI & Window Management` be split into smaller, more focused modules?**
  _Cohesion score 0.10960960960960961 - nodes in this community are weakly interconnected._
- **Should `VideoGLArea OpenGL & Graphics Offload` be split into smaller, more focused modules?**
  _Cohesion score 0.09538461538461539 - nodes in this community are weakly interconnected._