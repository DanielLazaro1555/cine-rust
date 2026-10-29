// window.rs
//
// Ventana principal de Cine Rust (`CineWindow`): conecta los módulos de área OpenGL
// (`VideoGLArea`), preferencias, lista de reproducción, historial, atajos,
// selector de archivos, selector de carpetas, URL, subtítulos y pistas de audio.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use gtk::{gdk, gio, glib};

use crate::ICON_NAME;
use crate::history::present_history_dialog;
use crate::mpris::MprisController;
use crate::mpv_gl_area::VideoGLArea;
use crate::options::PlaybackOptions;
use crate::playlist::{PlaylistItem, present_playlist_dialog};
use crate::preferences::SharedSettings;
use crate::shortcuts::present_shortcuts_dialog;
use crate::utils::{
    calculate_window_size, center_window_on_screen, collect_media_files, format_time,
    resolution_badge,
};

#[derive(Clone)]
pub struct CineWindow {
    pub window: adw::ApplicationWindow,
    toast_overlay: adw::ToastOverlay,
    video_area: VideoGLArea,
    start_page: adw::StatusPage,
    title_widget: adw::WindowTitle,
    controls_box: gtk::Box,
    play_pause_btn: gtk::Button,
    quality_menu_btn: gtk::MenuButton,
    progress_adj: gtk::Adjustment,
    time_elapsed_label: gtk::Label,
    time_total_label: gtk::Label,
    playlist_items: Rc<RefCell<Vec<PlaylistItem>>>,
    current_index: Rc<Cell<usize>>,
    last_video_dims: Rc<Cell<(i32, i32)>>,
    pub options: Rc<RefCell<PlaybackOptions>>,
    pub settings: SharedSettings,
    pub mpris: MprisController,
    inhibit_cookie: Rc<Cell<u32>>,
    is_playing: Rc<Cell<bool>>,
    updating_slider: Rc<Cell<bool>>,
}

impl CineWindow {
    pub fn new(
        app: &adw::Application,
        settings: SharedSettings,
        mpris: MprisController,
    ) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Cine Rust")
            .default_width(1024)
            .default_height(620)
            .width_request(332)
            .height_request(187)
            .build();

        gtk::WindowGroup::new().add_window(&window);

        // Cargar CSS personalizado de Cine desde el recurso embebido
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_resource("/io/github/diegopvlk/Cine/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let toast_overlay = adw::ToastOverlay::new();
        let video_overlay = gtk::Overlay::new();
        video_overlay.add_css_class("video-overlay");

        // Área OpenGL modularizada en `mpv_gl_area.rs` + `mpv_engine.rs`
        let video_area = VideoGLArea::new(
            settings.borrow().graphics_offload,
            settings.borrow().hwdec,
        );
        video_area.set_visible(true);
        video_overlay.set_child(Some(video_area.widget()));

        // Título de la barra superior
        let title_widget = adw::WindowTitle::new("Cine Rust", "");
        title_widget.set_visible(false);

        let headerbar = adw::HeaderBar::builder()
            .valign(gtk::Align::Start)
            .title_widget(&title_widget)
            .build();
        headerbar.add_css_class("osd");
        headerbar.add_css_class("header-bar");

        // Menú desplegable "Abrir" en la esquina superior izquierda (igual que el original)
        let open_menu = gio::Menu::new();
        let sec_open = gio::Menu::new();
        sec_open.append(Some("Abrir archivos…"), Some("win.clear-and-add"));
        sec_open.append(Some("Abrir carpeta…"), Some("win.open-folder"));
        sec_open.append(Some("Abrir URL…"), Some("win.open-url"));
        open_menu.append_section(None, &sec_open);

        let sec_add = gio::Menu::new();
        sec_add.append(Some("Añadir archivos a la lista…"), Some("win.add-playlist-files"));
        sec_add.append(Some("Añadir carpeta a la lista…"), Some("win.add-playlist-folder"));
        sec_add.append(Some("Añadir URL a la lista…"), Some("win.add-url"));
        open_menu.append_section(None, &sec_add);

        let sec_tracks = gio::Menu::new();
        sec_tracks.append(Some("Añadir pista de subtítulos…"), Some("win.add-sub-tracks"));
        sec_tracks.append(Some("Añadir pista de audio…"), Some("win.add-audio-tracks"));
        open_menu.append_section(None, &sec_tracks);

        let sec_hist = gio::Menu::new();
        sec_hist.append(Some("Historial de reproducción"), Some("win.open-history"));
        open_menu.append_section(None, &sec_hist);

        let open_menu_btn = gtk::MenuButton::builder()
            .label("Abrir")
            .always_show_arrow(true)
            .valign(gtk::Align::Center)
            .menu_model(&open_menu)
            .build();
        headerbar.pack_start(&open_menu_btn);

        // Menú principal en la esquina superior derecha
        let primary_menu = gio::Menu::new();
        let section_main = gio::Menu::new();
        section_main.append(Some("Nueva ventana"), Some("app.new-window"));
        section_main.append(Some("Preferencias"), Some("app.preferences"));
        section_main.append(Some("Atajos de teclado"), Some("win.custom-shortcuts"));
        section_main.append(Some("Acerca de Cine Rust"), Some("app.about"));
        section_main.append(Some("Salir"), Some("app.quit"));
        primary_menu.append_section(None, &section_main);

        let primary_menu_btn = gtk::MenuButton::builder()
            .valign(gtk::Align::Center)
            .primary(true)
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menú principal")
            .menu_model(&primary_menu)
            .build();
        headerbar.pack_end(&primary_menu_btn);

        // Página de inicio (AdwStatusPage) con todos los botones del diseño original
        let start_page = adw::StatusPage::builder()
            .margin_top(48)
            .valign(gtk::Align::Center)
            .halign(gtk::Align::Center)
            .icon_name(ICON_NAME)
            .title("Cine Rust")
            .description("Arrastra y suelta archivos o carpetas aquí")
            .build();
        start_page.add_css_class("start-page");

        let start_buttons_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();

        let top_open_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        top_open_row.add_css_class("open-btns-box");
        let open_btn = gtk::Button::builder()
            .label("Abrir…")
            .action_name("win.clear-and-add")
            .hexpand(true)
            .build();
        open_btn.add_css_class("suggested-action");
        open_btn.add_css_class("pill");

        let history_btn = gtk::Button::builder()
            .tooltip_text("Historial de reproducción")
            .icon_name("document-open-recent-symbolic")
            .action_name("win.open-history")
            .build();
        history_btn.add_css_class("suggested-action");
        history_btn.add_css_class("circular");
        history_btn.add_css_class("watch-history");

        top_open_row.append(&open_btn);
        top_open_row.append(&history_btn);
        start_buttons_box.append(&top_open_row);

        let open_folder_btn = gtk::Button::builder()
            .label("Abrir carpeta")
            .action_name("win.open-folder")
            .build();
        open_folder_btn.add_css_class("pill");
        start_buttons_box.append(&open_folder_btn);

        let open_url_btn = gtk::Button::builder()
            .label("Abrir URL")
            .action_name("win.open-url")
            .build();
        open_url_btn.add_css_class("pill");
        start_buttons_box.append(&open_url_btn);

        start_page.set_child(Some(&start_buttons_box));

        // Barra inferior de controles (OSD)
        let controls_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .valign(gtk::Align::End)
            .spacing(4)
            .margin_start(16)
            .margin_end(16)
            .margin_bottom(12)
            .visible(false)
            .build();
        controls_box.add_css_class("osd");
        controls_box.add_css_class("controls");

        let prev_btn = gtk::Button::builder()
            .icon_name("cine-skip-backward-symbolic")
            .tooltip_text("Anterior")
            .build();
        prev_btn.add_css_class("circular");
        prev_btn.add_css_class("flat");

        let play_pause_btn = gtk::Button::builder()
            .icon_name("cine-playback-start-symbolic")
            .tooltip_text("Reproducir/Pausar (Espacio)")
            .build();
        play_pause_btn.add_css_class("circular");
        play_pause_btn.add_css_class("flat");

        let next_btn = gtk::Button::builder()
            .icon_name("cine-skip-forward-symbolic")
            .tooltip_text("Siguiente")
            .build();
        next_btn.add_css_class("circular");
        next_btn.add_css_class("flat");

        // Control de volumen desplegable
        let vol_adj = gtk::Adjustment::new(100.0, 0.0, 150.0, 1.0, 5.0, 0.0);
        let vol_scale = gtk::Scale::builder()
            .orientation(gtk::Orientation::Horizontal)
            .adjustment(&vol_adj)
            .width_request(160)
            .draw_value(false)
            .build();
        let vol_pop = gtk::Popover::builder().child(&vol_scale).build();
        let volume_menu_btn = gtk::MenuButton::builder()
            .icon_name("cine-volume-max-symbolic")
            .tooltip_text("Volumen")
            .popover(&vol_pop)
            .direction(gtk::ArrowType::Up)
            .build();
        volume_menu_btn.add_css_class("circular");
        volume_menu_btn.add_css_class("flat");

        let playlist_bar_btn = gtk::Button::builder()
            .icon_name("cine-playlist-symbolic")
            .tooltip_text("Lista de reproducción (Ctrl+P)")
            .action_name("win.open-playlist-dialog")
            .build();
        playlist_bar_btn.add_css_class("circular");
        playlist_bar_btn.add_css_class("flat");

        // Menú selector de Calidad de Video (4K / 1440p / 1080p / 720p / 480p)
        let quality_menu = gio::Menu::new();
        quality_menu.append(Some("Máxima disponible (Auto / 4K)"), Some("win.set-quality('best')"));
        quality_menu.append(Some("1440p (2K QHD)"), Some("win.set-quality('1440')"));
        quality_menu.append(Some("1080p (Full HD)"), Some("win.set-quality('1080')"));
        quality_menu.append(Some("720p (HD)"), Some("win.set-quality('720')"));
        quality_menu.append(Some("480p (SD)"), Some("win.set-quality('480')"));

        let quality_menu_btn = gtk::MenuButton::builder()
            .label("Calidad")
            .tooltip_text("Calidad de video actual y selector de resolución")
            .menu_model(&quality_menu)
            .direction(gtk::ArrowType::Up)
            .build();
        quality_menu_btn.add_css_class("flat");

        let fullscreen_btn = gtk::Button::builder()
            .icon_name("cine-view-fullscreen-symbolic")
            .tooltip_text("Pantalla completa (f)")
            .build();
        fullscreen_btn.add_css_class("circular");
        fullscreen_btn.add_css_class("flat");

        let progress_adj = gtk::Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 0.0);
        let progress_scale = gtk::Scale::builder()
            .orientation(gtk::Orientation::Horizontal)
            .adjustment(&progress_adj)
            .hexpand(true)
            .draw_value(false)
            .build();

        let time_elapsed_label = gtk::Label::new(Some(&format_time(0.0)));
        time_elapsed_label.add_css_class("numeric");
        let time_total_label = gtk::Label::new(Some(&format_time(0.0)));
        time_total_label.add_css_class("numeric");

        let time_row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_start(10)
            .margin_end(10)
            .margin_top(6)
            .margin_bottom(6)
            .build();
        time_row.append(&prev_btn);
        time_row.append(&play_pause_btn);
        time_row.append(&next_btn);
        time_row.append(&volume_menu_btn);
        time_row.append(&progress_scale);
        time_row.append(&time_elapsed_label);
        time_row.append(&gtk::Label::new(Some("/")));
        time_row.append(&time_total_label);
        time_row.append(&quality_menu_btn);
        time_row.append(&playlist_bar_btn);
        time_row.append(&fullscreen_btn);
        controls_box.append(&time_row);

        // Contenedor vertical para HeaderBar arriba y controles abajo
        let ui_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        ui_box.append(&headerbar);
        let spacer = gtk::Separator::new(gtk::Orientation::Horizontal);
        spacer.set_vexpand(true);
        spacer.set_opacity(0.0);
        spacer.set_can_target(false);
        ui_box.append(&spacer);
        ui_box.append(&controls_box);

        video_overlay.add_overlay(&ui_box);
        video_overlay.add_overlay(&start_page);

        let window_handle = gtk::WindowHandle::builder()
            .child(&video_overlay)
            .build();
        toast_overlay.set_child(Some(&window_handle));
        window.set_content(Some(&toast_overlay));

        let cine_win = Self {
            window: window.clone(),
            toast_overlay,
            video_area,
            start_page,
            title_widget,
            controls_box,
            play_pause_btn: play_pause_btn.clone(),
            quality_menu_btn: quality_menu_btn.clone(),
            progress_adj: progress_adj.clone(),
            time_elapsed_label,
            time_total_label,
            playlist_items: Rc::new(RefCell::new(Vec::new())),
            current_index: Rc::new(Cell::new(0)),
            last_video_dims: Rc::new(Cell::new((0, 0))),
            options: Rc::new(RefCell::new(PlaybackOptions::default())),
            settings,
            mpris,
            inhibit_cookie: Rc::new(Cell::new(0)),
            is_playing: Rc::new(Cell::new(false)),
            updating_slider: Rc::new(Cell::new(false)),
        };

        // Botón Play/Pause
        {
            let this = cine_win.clone();
            play_pause_btn.connect_clicked(move |_| {
                this.toggle_pause();
            });
        }

        // Botón Anterior / Siguiente en la playlist
        {
            let this = cine_win.clone();
            prev_btn.connect_clicked(move |_| {
                this.play_prev();
            });
        }
        {
            let this = cine_win.clone();
            next_btn.connect_clicked(move |_| {
                this.play_next();
            });
        }

        // Volumen
        {
            let this = cine_win.clone();
            vol_scale.connect_value_changed(move |scale| {
                this.video_area.set_volume(scale.value());
            });
        }

        // Botón Pantalla Completa
        {
            let win = window.clone();
            fullscreen_btn.connect_clicked(move |_| {
                if win.is_fullscreen() {
                    win.unfullscreen();
                } else {
                    win.fullscreen();
                }
            });
        }

        // Barra de progreso interactiva (Seek)
        {
            let this = cine_win.clone();
            progress_scale.connect_value_changed(move |scale| {
                if !this.updating_slider.get() {
                    this.video_area.seek_absolute(scale.value());
                }
            });
        }

        cine_win.setup_actions();
        cine_win.setup_keyboard_and_drop(&video_overlay);
        cine_win.setup_progress_timer();
        cine_win.setup_close_request();

        cine_win
    }

    pub fn present(&self) {
        self.window.present();
        glib::timeout_add_local_once(Duration::from_millis(80), || {
            center_window_on_screen(None);
        });
    }

    pub fn show_toast(&self, msg: &str) {
        let toast = adw::Toast::builder().title(msg).timeout(2).build();
        self.toast_overlay.add_toast(toast);
    }

    pub fn toggle_pause(&self) {
        let paused = self.video_area.toggle_pause();
        self.is_playing.set(!paused);
        self.play_pause_btn.set_icon_name(if paused {
            "cine-playback-start-symbolic"
        } else {
            "cine-playback-pause-symbolic"
        });
        self.sync_inhibit();
    }

    pub fn play_prev(&self) {
        let idx = self.current_index.get();
        if idx > 0 {
            self.play_index(idx - 1);
        }
    }

    pub fn play_next(&self) {
        let len = self.playlist_items.borrow().len();
        let idx = self.current_index.get();
        if idx + 1 < len {
            self.play_index(idx + 1);
        }
    }

    fn play_index(&self, new_idx: usize) {
        let mut items = self.playlist_items.borrow_mut();
        if new_idx >= items.len() {
            return;
        }
        for (i, item) in items.iter_mut().enumerate() {
            item.playing = i == new_idx;
        }
        let target_path = items[new_idx].path.to_string_lossy().into_owned();
        let target_title = items[new_idx].title.clone();
        drop(items);

        self.current_index.set(new_idx);
        self.last_video_dims.set((0, 0));
        self.video_area.load_file(&target_path, "replace");
        self.window.set_title(Some(&target_title));
        self.title_widget.set_title(&target_title);
    }

    /// Redimensiona la ventana automáticamente según las dimensiones reales del video (`vid_w` x `vid_h`)
    /// si la opción `auto_resize_window` está activa en Preferencias.
    pub fn apply_video_window_size(&self, vid_w: i32, vid_h: i32) {
        if !self.settings.borrow().auto_resize_window {
            return;
        }
        if self.window.is_fullscreen() || self.window.is_maximized() {
            return;
        }

        let mut mon_w = 1920;
        let mut mon_h = 1080;
        if let Some(surface) = self.window.surface() {
            if let Some(display) = gdk::Display::default() {
                if let Some(monitor) = display.monitor_at_surface(&surface) {
                    let geom = monitor.geometry();
                    mon_w = geom.width();
                    mon_h = geom.height();
                }
            }
        }

        let (target_w, target_h) = calculate_window_size(vid_w, vid_h, mon_w, mon_h);
        self.window.set_default_size(target_w, target_h);
        center_window_on_screen(Some((target_w, target_h)));
        glib::timeout_add_local_once(Duration::from_millis(150), move || {
            center_window_on_screen(Some((target_w, target_h)));
        });
    }

    pub fn sync_inhibit(&self) {
        let Some(app) = self.window.application() else {
            return;
        };
        if !app.is_registered() {
            self.inhibit_cookie.set(0);
            return;
        }

        let should_inhibit = self.is_playing.get();
        let current_cookie = self.inhibit_cookie.get();

        if should_inhibit && current_cookie == 0 {
            let cookie = app.inhibit(
                Some(&self.window),
                gtk::ApplicationInhibitFlags::IDLE,
                Some("Playing Media"),
            );
            self.inhibit_cookie.set(cookie);
        } else if !should_inhibit && current_cookie != 0 {
            app.uninhibit(current_cookie);
            self.inhibit_cookie.set(0);
        }
    }

    fn setup_progress_timer(&self) {
        let this = self.clone();
        glib::timeout_add_local(Duration::from_millis(250), move || {
            if let Some(dims) = this.video_area.video_dimensions() {
                if dims != this.last_video_dims.get() {
                    this.last_video_dims.set(dims);
                    this.apply_video_window_size(dims.0, dims.1);
                }
            }

            let (q_info, media_title) = this.video_area.video_quality_info();
            if let Some(title) = media_title {
                if !title.is_empty() && this.title_widget.title() != title {
                    this.window.set_title(Some(&title));
                    this.title_widget.set_title(&title);
                }
            }
            if let Some((w, h, fps, codec, hwdec)) = q_info {
                let badge = resolution_badge(w, h);
                let short_badge = badge.split_whitespace().next().unwrap_or("HD");
                this.quality_menu_btn.set_label(short_badge);
                let subtitle = if fps > 0.0 {
                    format!("{badge} · {fps:.0} FPS · {codec} · GPU: {hwdec}")
                } else {
                    format!("{badge} · {codec} · GPU: {hwdec}")
                };
                this.title_widget.set_subtitle(&subtitle);
            }

            let (pos, dur, paused) = this.video_area.time_pos_and_duration();
            if dur > 0.0 {
                this.updating_slider.set(true);
                this.progress_adj.set_upper(dur);
                this.progress_adj.set_value(pos);
                this.updating_slider.set(false);
                this.time_elapsed_label.set_label(&format_time(pos));
                this.time_total_label.set_label(&format_time(dur));
                this.is_playing.set(!paused);
                this.play_pause_btn.set_icon_name(if paused {
                    "cine-playback-start-symbolic"
                } else {
                    "cine-playback-pause-symbolic"
                });
            }
            glib::ControlFlow::Continue
        });
    }

    fn setup_keyboard_and_drop(&self, overlay: &gtk::Overlay) {
        let key_ctrl = gtk::EventControllerKey::new();
        // Fase Capture para interceptar Espacio/flechas antes de que botones o sliders los consuman
        key_ctrl.set_propagation_phase(gtk::PropagationPhase::Capture);
        let this = self.clone();
        key_ctrl.connect_key_pressed(move |_, key, _, _| {
            // Si hay un diálogo abierto (por ejemplo "Abrir URL"), permitir escribir normalmente
            if this.window.visible_dialog().is_some() {
                return glib::Propagation::Proceed;
            }
            if let Some(focus) = GtkWindowExt::focus(&this.window) {
                if focus.is::<gtk::Text>() || focus.is::<gtk::Entry>() {
                    return glib::Propagation::Proceed;
                }
            }

            match key {
                gdk::Key::space | gdk::Key::k | gdk::Key::K => {
                    this.toggle_pause();
                    glib::Propagation::Stop
                }
                gdk::Key::Left => {
                    this.video_area.seek_relative(-5.0);
                    glib::Propagation::Stop
                }
                gdk::Key::Right => {
                    this.video_area.seek_relative(5.0);
                    glib::Propagation::Stop
                }
                gdk::Key::f | gdk::Key::F | gdk::Key::F11 => {
                    if this.window.is_fullscreen() {
                        this.window.unfullscreen();
                    } else {
                        this.window.fullscreen();
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::i | gdk::Key::I => {
                    this.video_area.toggle_stats();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        self.window.add_controller(key_ctrl);

        let drop_target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
        let this = self.clone();
        drop_target.connect_drop(move |_, value, _, _| {
            if let Ok(file_list) = value.get::<gdk::FileList>() {
                let files = file_list.files();
                this.open_files(&files, true);
                return true;
            }
            false
        });
        overlay.add_controller(drop_target);
    }

    fn setup_close_request(&self) {
        let this = self.clone();
        self.window.connect_close_request(move |_| {
            this.is_playing.set(false);
            this.sync_inhibit();
            this.video_area.shutdown();
            glib::Propagation::Proceed
        });
    }

    fn setup_actions(&self) {
        // win.quit (tecla 'q' o Ctrl+W)
        let action_quit = gio::SimpleAction::new("quit", None);
        let win_weak = self.window.downgrade();
        action_quit.connect_activate(move |_, _| {
            if let Some(win) = win_weak.upgrade() {
                win.close();
            }
        });
        self.window.add_action(&action_quit);

        // win.clear-and-add (Ctrl+O): Abrir archivos reemplazando lista
        let action_open = gio::SimpleAction::new("clear-and-add", None);
        let this = self.clone();
        action_open.connect_activate(move |_, _| {
            this.open_file_dialog(true);
        });
        self.window.add_action(&action_open);

        // win.add-playlist-files (Ctrl+Shift+O): Añadir archivos a la lista
        let action_add_files = gio::SimpleAction::new("add-playlist-files", None);
        let this = self.clone();
        action_add_files.connect_activate(move |_, _| {
            this.open_file_dialog(false);
        });
        self.window.add_action(&action_add_files);

        // win.open-folder (Ctrl+I): Abrir carpeta completa
        let action_folder = gio::SimpleAction::new("open-folder", None);
        let this = self.clone();
        action_folder.connect_activate(move |_, _| {
            this.open_folder_dialog(true);
        });
        self.window.add_action(&action_folder);

        // win.add-playlist-folder (Ctrl+Shift+I): Añadir carpeta a la lista
        let action_add_folder = gio::SimpleAction::new("add-playlist-folder", None);
        let this = self.clone();
        action_add_folder.connect_activate(move |_, _| {
            this.open_folder_dialog(false);
        });
        self.window.add_action(&action_add_folder);

        // win.open-url (Ctrl+U) y win.add-url (Ctrl+Shift+U)
        let action_url = gio::SimpleAction::new("open-url", None);
        let this = self.clone();
        action_url.connect_activate(move |_, _| {
            this.open_url_dialog(true);
        });
        self.window.add_action(&action_url);

        let action_add_url = gio::SimpleAction::new("add-url", None);
        let this = self.clone();
        action_add_url.connect_activate(move |_, _| {
            this.open_url_dialog(false);
        });
        self.window.add_action(&action_add_url);

        // win.add-sub-tracks: Añadir subtítulos externos
        let action_sub = gio::SimpleAction::new("add-sub-tracks", None);
        let this = self.clone();
        action_sub.connect_activate(move |_, _| {
            this.add_external_track_dialog("sub-add", "Seleccionar archivo de subtítulos");
        });
        self.window.add_action(&action_sub);

        // win.add-audio-tracks: Añadir audio externo
        let action_aud = gio::SimpleAction::new("add-audio-tracks", None);
        let this = self.clone();
        action_aud.connect_activate(move |_, _| {
            this.add_external_track_dialog("audio-add", "Seleccionar pista de audio");
        });
        self.window.add_action(&action_aud);

        // win.open-history (Ctrl+H)
        let action_history = gio::SimpleAction::new("open-history", None);
        let win_weak = self.window.downgrade();
        action_history.connect_activate(move |_, _| {
            if let Some(win) = win_weak.upgrade() {
                present_history_dialog(&win);
            }
        });
        self.window.add_action(&action_history);

        // win.open-playlist-dialog (Ctrl+P)
        let action_playlist = gio::SimpleAction::new("open-playlist-dialog", None);
        let this = self.clone();
        action_playlist.connect_activate(move |_, _| {
            present_playlist_dialog(&this.window, &this.playlist_items.borrow());
        });
        self.window.add_action(&action_playlist);

        // win.custom-shortcuts (Ctrl+?)
        let action_shortcuts = gio::SimpleAction::new("custom-shortcuts", None);
        let win_weak = self.window.downgrade();
        action_shortcuts.connect_activate(move |_, _| {
            if let Some(win) = win_weak.upgrade() {
                present_shortcuts_dialog(&win);
            }
        });
        self.window.add_action(&action_shortcuts);

        // win.set-quality: Cambiar calidad de stream (4K / 1440p / 1080p / 720p / 480p)
        let action_quality =
            gio::SimpleAction::new("set-quality", Some(glib::VariantTy::STRING));
        let this = self.clone();
        action_quality.connect_activate(move |_, param| {
            let Some(tier) = param.and_then(|p| p.str()) else {
                return;
            };
            let fmt = match tier {
                "1440" => "bestvideo[height<=1440]+bestaudio/best[height<=1440]",
                "1080" => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
                "720" => "bestvideo[height<=720]+bestaudio/best[height<=720]",
                "480" => "bestvideo[height<=480]+bestaudio/best[height<=480]",
                _ => "bestvideo+bestaudio/best",
            };
            this.video_area.set_ytdl_format(fmt);
            this.last_video_dims.set((0, 0));

            let (pos, _, _) = this.video_area.time_pos_and_duration();
            let cur_idx = this.current_index.get();
            let cur_path = this
                .playlist_items
                .borrow()
                .get(cur_idx)
                .map(|i| i.path.to_string_lossy().into_owned());
            if let Some(path) = cur_path {
                this.video_area.load_file(&path, "replace");
                if pos > 1.0 {
                    let va = this.video_area.clone();
                    glib::timeout_add_local_once(Duration::from_millis(1200), move || {
                        va.seek_absolute(pos);
                    });
                }
                this.show_toast(&format!("Calidad cambiada a {tier}p"));
            }
        });
        self.window.add_action(&action_quality);
    }

#[allow(deprecated)]
    fn open_file_dialog(&self, clear_existing: bool) {
        let dialog = gtk::FileChooserDialog::new(
            Some(if clear_existing {
                "Abrir archivos de video"
            } else {
                "Añadir archivos a la lista"
            }),
            Some(&self.window),
            gtk::FileChooserAction::Open,
            &[
                ("Cancelar", gtk::ResponseType::Cancel),
                (
                    if clear_existing { "Abrir" } else { "Añadir" },
                    gtk::ResponseType::Accept,
                ),
            ],
        );
        dialog.set_modal(true);
        dialog.set_select_multiple(true);

        let this = self.clone();
        dialog.connect_response(move | dlg, response | {
            if response == gtk::ResponseType::Accept {
                let list = dlg.files();
                let mut files = Vec::new();
                for i in 0..list.n_items() {
                    if let Some(file) = list.item(i).and_downcast::<gio::File>() {
                        files.push(file);
                    }
                }
                this.open_files(&files, clear_existing);
            }
            dlg.destroy();
        });
        dialog.present();
    }

    #[allow(deprecated)]
    fn open_folder_dialog(&self, clear_existing: bool) {
        let dialog = gtk::FileChooserDialog::new(
            Some(if clear_existing {
                "Abrir carpeta de videos"
            } else {
                "Añadir carpeta a la lista"
            }),
            Some(&self.window),
            gtk::FileChooserAction::SelectFolder,
            &[
                ("Cancelar", gtk::ResponseType::Cancel),
                (
                    if clear_existing {
                        "Abrir carpeta"
                    } else {
                        "Añadir carpeta"
                    },
                    gtk::ResponseType::Accept,
                ),
            ],
        );
        dialog.set_modal(true);

        let this = self.clone();
        dialog.connect_response(move |dlg, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(folder_file) = dlg.file() {
                    this.open_files(&[folder_file], clear_existing);
                }
            }
            dlg.destroy();
        });
        dialog.present();
    }

    fn open_url_dialog(&self, clear_existing: bool) {
        let dialog = adw::AlertDialog::builder()
            .heading(if clear_existing {
                "Abrir URL"
            } else {
                "Añadir URL a la lista"
            })
            .body("Introduce la dirección URL del video o transmisión:")
            .default_response("open")
            .close_response("cancel")
            .build();

        dialog.add_response("cancel", "Cancelar");
        dialog.add_response("open", if clear_existing { "Abrir" } else { "Añadir" });
        dialog.set_response_appearance("open", adw::ResponseAppearance::Suggested);

        let entry = gtk::Entry::builder()
            .placeholder_text("https://...")
            .activates_default(true)
            .build();
        dialog.set_extra_child(Some(&entry));

        let this = self.clone();
        dialog.choose(
            &self.window,
            None::<&gio::Cancellable>,
            move |response| {
                if response == "open" {
                    let url = entry.text().trim().to_string();
                    if !url.is_empty() {
                        this.open_url(&url, clear_existing);
                    }
                }
            },
        );
    }

    #[allow(deprecated)]
    fn add_external_track_dialog(&self, mpv_cmd: &'static str, title: &str) {
        let dialog = gtk::FileChooserDialog::new(
            Some(title),
            Some(&self.window),
            gtk::FileChooserAction::Open,
            &[
                ("Cancelar", gtk::ResponseType::Cancel),
                ("Añadir", gtk::ResponseType::Accept),
            ],
        );
        dialog.set_modal(true);

        let this = self.clone();
        dialog.connect_response(move |dlg, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = dlg.file() {
                    if let Some(path) = file.path() {
                        if let Some(engine) = this.video_area.mpv.borrow().as_ref() {
                            engine.command(&[mpv_cmd, &path.to_string_lossy(), "select"]);
                            this.show_toast("Pista añadida");
                        }
                    }
                }
            }
            dlg.destroy();
        });
        dialog.present();
    }

    pub fn open_url(&self, url: &str, clear_existing: bool) {
        eprintln!("[CineRust] Abriendo URL: {url}");
        self.show_toast("Cargando URL con yt-dlp…");
        self.last_video_dims.set((0, 0));
        let mode = if clear_existing { "replace" } else { "append-play" };
        self.video_area.load_file(url, mode);

        let mut items = self.playlist_items.borrow_mut();
        if clear_existing {
            items.clear();
            self.current_index.set(0);
        }
        let is_first = items.is_empty();
        items.push(PlaylistItem {
            path: PathBuf::from(url),
            title: url.to_string(),
            playing: is_first,
        });
        drop(items);

        self.window.set_title(Some(url));
        self.title_widget.set_title(url);
        self.title_widget.set_visible(true);
        self.start_page.set_visible(false);
        self.controls_box.set_visible(true);
        self.is_playing.set(true);
        self.play_pause_btn
            .set_icon_name("cine-playback-pause-symbolic");
        self.sync_inhibit();
        center_window_on_screen(None);
    }

    pub fn open_files(&self, files: &[gio::File], clear_existing: bool) {
        let mut media_paths = Vec::new();
        for gfile in files {
            if let Some(path) = gfile.path() {
                let collected = collect_media_files(&path);
                if collected.is_empty() && path.is_file() {
                    media_paths.push(path);
                } else {
                    media_paths.extend(collected);
                }
            }
        }

        let Some(first_path) = media_paths.first().cloned() else {
            self.show_toast("No se encontraron archivos de video en la selección");
            return;
        };

        let mut items = self.playlist_items.borrow_mut();
        if clear_existing {
            items.clear();
            self.current_index.set(0);
        }

        let start_empty = items.is_empty();
        for (idx, path) in media_paths.iter().enumerate() {
            let title = path
                .file_name()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Video".to_string());
            let mode = if clear_existing && idx == 0 {
                "replace"
            } else {
                "append-play"
            };
            self.video_area.load_file(&path.to_string_lossy(), mode);
            items.push(PlaylistItem {
                path: path.clone(),
                title,
                playing: start_empty && idx == 0,
            });
        }
        let total_loaded = media_paths.len();
        drop(items);

        if clear_existing || start_empty {
            self.last_video_dims.set((0, 0));
            let display_name = first_path
                .file_name()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Video".to_string());
            self.window.set_title(Some(&display_name));
            self.title_widget.set_title(&display_name);
        }

        self.title_widget.set_visible(true);
        self.start_page.set_visible(false);
        self.controls_box.set_visible(true);
        self.is_playing.set(true);
        self.play_pause_btn
            .set_icon_name("cine-playback-pause-symbolic");
        self.sync_inhibit();
        self.mpris.update_metadata();

        if total_loaded > 1 {
            self.show_toast(&format!("{total_loaded} videos cargados en la lista"));
        }
    }
}
