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
use crate::history::{present_history_dialog, record_history_entry};
use crate::mpris::MprisController;
use crate::mpv_gl_area::VideoGLArea;
use crate::options::{PlaybackOptions, create_options_menu_button};
use crate::playlist::{PlaylistItem, present_playlist_dialog};
use crate::preferences::SharedSettings;
use crate::save_session::{load_playlist_session, save_playlist_session};
use crate::shortcuts::present_shortcuts_dialog;
use crate::tracks::{create_audio_menu_button, create_subtitles_menu_button};
use crate::utils::{
    calculate_window_size, center_active_cine_window, collect_media_files, format_time,
    resolution_badge,
};

#[derive(Clone)]
pub struct CineWindow {
    pub window: adw::ApplicationWindow,
    toast_overlay: adw::ToastOverlay,
    video_area: VideoGLArea,
    start_page: adw::StatusPage,
    headerbar: adw::HeaderBar,
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
    loop_mode: Rc<Cell<u8>>,
    shuffle_enabled: Rc<Cell<bool>>,
    autohide_timer_id: Rc<RefCell<Option<glib::SourceId>>>,
    floating_lock_btn: gtk::Button,
    is_locked: Rc<Cell<bool>>,
    lock_timer_id: Rc<RefCell<Option<glib::SourceId>>>,
    max_detected_height: Rc<Cell<i32>>,
}

impl CineWindow {
    pub fn new(
        app: &adw::Application,
        settings: SharedSettings,
        mpris: MprisController,
    ) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Cine (Rust)")
            .default_width(1024)
            .default_height(620)
            .width_request(360)
            .height_request(202)
            .build();

        gtk::WindowGroup::new().add_window(&window);

        // Incluir el CSS directamente para que cada compilación use los estilos actuales.
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_bytes(&glib::Bytes::from_static(include_bytes!(
            "../resources/style.css"
        )));
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
        let title_widget = adw::WindowTitle::new("Cine (Rust)", "");
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
            .title("Cine (Rust)")
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
            .margin_start(8)
            .margin_end(8)
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

        // Menú selector de Calidad de Video (Automático por defecto; se adapta dinámicamente al video real)
        let quality_menu = gio::Menu::new();
        quality_menu.append(Some("Automático (Calidad óptima del video)"), Some("win.set-quality('best')"));

        let quality_menu_btn = gtk::MenuButton::builder()
            .label("Auto")
            .tooltip_text("Calidad de video (Automático por defecto)")
            .menu_model(&quality_menu)
            .direction(gtk::ArrowType::Up)
            .build();
        quality_menu_btn.add_css_class("flat");

        // Selectores interactivos de pistas de audio y subtítulos
        let toast_overlay_weak = toast_overlay.downgrade();
        let toast_notifier = move |msg: &str| {
            if let Some(ov) = toast_overlay_weak.upgrade() {
                let toast = adw::Toast::builder().title(msg).timeout(2).build();
                ov.add_toast(toast);
            }
        };

        let add_audio_cb = {
            let win_weak = window.downgrade();
            let va = video_area.clone();
            let t_notify = toast_notifier.clone();
            move || {
                if let Some(win) = win_weak.upgrade() {
                    Self::open_track_file_dialog(&win, &va, "audio-add", "Añadir pista de audio externa", t_notify.clone());
                }
            }
        };

        let add_sub_cb = {
            let win_weak = window.downgrade();
            let va = video_area.clone();
            let t_notify = toast_notifier.clone();
            move || {
                if let Some(win) = win_weak.upgrade() {
                    Self::open_track_file_dialog(&win, &va, "sub-add", "Añadir archivo de subtítulos", t_notify.clone());
                }
            }
        };

        let audio_menu_btn = create_audio_menu_button(video_area.clone(), add_audio_cb);
        let subtitles_menu_btn = create_subtitles_menu_button(video_area.clone(), add_sub_cb);

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

        // Botón de opciones de video y transformación
        let options = Rc::new(RefCell::new(PlaybackOptions::default()));
        let options_menu_btn = create_options_menu_button(video_area.clone(), options.clone());

        // Botón de reproducción aleatoria (Shuffle)
        let shuffle_btn = gtk::Button::builder()
            .icon_name("cine-playlist-shuffle-symbolic")
            .tooltip_text("Modo aleatorio")
            .build();
        shuffle_btn.add_css_class("circular");
        shuffle_btn.add_css_class("flat");

        // Botón de repetición (Loop)
        let repeat_btn = gtk::Button::builder()
            .icon_name("cine-playlist-repeat-symbolic")
            .tooltip_text("Repetir: Desactivado")
            .build();
        repeat_btn.add_css_class("circular");
        repeat_btn.add_css_class("flat");

        // Botón de bloqueo (candado) en barra inferior
        let lock_btn = gtk::Button::builder()
            .icon_name("changes-prevent-symbolic")
            .tooltip_text("Bloquear controles (L)")
            .build();
        lock_btn.add_css_class("circular");
        lock_btn.add_css_class("flat");

        // Botón candado flotante sobre el video (se muestra durante 5 segundos si se mueve el cursor)
        let floating_lock_btn = gtk::Button::builder()
            .icon_name("changes-prevent-symbolic")
            .tooltip_text("Desbloquear controles (L)")
            .halign(gtk::Align::End)
            .valign(gtk::Align::Start)
            .margin_top(16)
            .margin_end(16)
            .visible(false)
            .build();
        floating_lock_btn.add_css_class("circular");
        floating_lock_btn.add_css_class("osd");
        floating_lock_btn.add_css_class("suggested-action");

        let progress_row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .hexpand(true)
            .margin_start(8)
            .margin_end(8)
            .margin_top(4)
            .margin_bottom(0)
            .build();
        progress_row.append(&progress_scale);

        let time_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(3)
            .margin_start(4)
            .valign(gtk::Align::Center)
            .build();
        time_box.append(&time_elapsed_label);
        let time_sep = gtk::Label::new(Some("/"));
        time_sep.add_css_class("dim-label");
        time_box.append(&time_sep);
        time_box.append(&time_total_label);

        // FlowBox mueve los controles a otra línea cuando la ventana se estrecha.
        let buttons_row = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .row_spacing(2)
            .column_spacing(2)
            .homogeneous(false)
            .halign(gtk::Align::Fill)
            .margin_start(6)
            .margin_end(6)
            .margin_bottom(4)
            .build();
        buttons_row.add_css_class("controls-wrap");

        // Cada botón puede pasar individualmente a la siguiente línea, sin reservar
        // espacio para un grupo completo que ya no cabe.
        for control in [
            prev_btn.upcast_ref::<gtk::Widget>(),
            play_pause_btn.upcast_ref::<gtk::Widget>(),
            next_btn.upcast_ref::<gtk::Widget>(),
            volume_menu_btn.upcast_ref::<gtk::Widget>(),
            time_box.upcast_ref::<gtk::Widget>(),
            quality_menu_btn.upcast_ref::<gtk::Widget>(),
            audio_menu_btn.upcast_ref::<gtk::Widget>(),
            subtitles_menu_btn.upcast_ref::<gtk::Widget>(),
            options_menu_btn.upcast_ref::<gtk::Widget>(),
            shuffle_btn.upcast_ref::<gtk::Widget>(),
            repeat_btn.upcast_ref::<gtk::Widget>(),
            lock_btn.upcast_ref::<gtk::Widget>(),
            playlist_bar_btn.upcast_ref::<gtk::Widget>(),
            fullscreen_btn.upcast_ref::<gtk::Widget>(),
        ] {
            buttons_row.append(control);
        }

        controls_box.append(&progress_row);
        controls_box.append(&buttons_row);

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
        video_overlay.add_overlay(&floating_lock_btn);
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
            headerbar: headerbar.clone(),
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
            options,
            settings: settings.clone(),
            mpris,
            inhibit_cookie: Rc::new(Cell::new(0)),
            is_playing: Rc::new(Cell::new(false)),
            updating_slider: Rc::new(Cell::new(false)),
            loop_mode: Rc::new(Cell::new(0)),
            shuffle_enabled: Rc::new(Cell::new(false)),
            autohide_timer_id: Rc::new(RefCell::new(None)),
            floating_lock_btn: floating_lock_btn.clone(),
            is_locked: Rc::new(Cell::new(false)),
            lock_timer_id: Rc::new(RefCell::new(None)),
            max_detected_height: Rc::new(Cell::new(0)),
        };

        // Clic en candado de controles para bloquear
        {
            let this = cine_win.clone();
            lock_btn.connect_clicked(move |_| {
                this.lock_controls();
            });
        }

        // Clic en candado flotante para desbloquear
        {
            let this = cine_win.clone();
            floating_lock_btn.connect_clicked(move |_| {
                this.unlock_controls();
            });
        }

        // Botón Shuffle (aleatorio)
        {
            let this = cine_win.clone();
            let s_btn = shuffle_btn.clone();
            shuffle_btn.connect_clicked(move |_| {
                let current = this.shuffle_enabled.get();
                let new_val = !current;
                this.shuffle_enabled.set(new_val);
                if new_val {
                    s_btn.add_css_class("suggested-action");
                    this.show_toast("Modo aleatorio activado");
                } else {
                    s_btn.remove_css_class("suggested-action");
                    this.show_toast("Modo aleatorio desactivado");
                }
            });
        }

        // Botón Repeat (bucle de archivo o lista)
        {
            let this = cine_win.clone();
            let r_btn = repeat_btn.clone();
            repeat_btn.connect_clicked(move |_| {
                let next_mode = (this.loop_mode.get() + 1) % 3;
                this.loop_mode.set(next_mode);
                match next_mode {
                    1 => {
                        r_btn.set_icon_name("cine-repeat-file-symbolic");
                        r_btn.add_css_class("suggested-action");
                        r_btn.set_tooltip_text(Some("Repetir: Archivo actual"));
                        this.video_area.set_loop_file(true);
                        this.video_area.set_loop_playlist(false);
                        this.show_toast("Repetir: Archivo actual");
                    }
                    2 => {
                        r_btn.set_icon_name("cine-playlist-repeat-symbolic");
                        r_btn.add_css_class("suggested-action");
                        r_btn.set_tooltip_text(Some("Repetir: Toda la lista"));
                        this.video_area.set_loop_file(false);
                        this.video_area.set_loop_playlist(true);
                        this.show_toast("Repetir: Toda la lista");
                    }
                    _ => {
                        r_btn.set_icon_name("cine-playlist-repeat-symbolic");
                        r_btn.remove_css_class("suggested-action");
                        r_btn.set_tooltip_text(Some("Repetir: Desactivado"));
                        this.video_area.set_loop_file(false);
                        this.video_area.set_loop_playlist(false);
                        this.show_toast("Repetición desactivada");
                    }
                }
            });
        }

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
        cine_win.setup_mouse_gestures(&video_overlay);
        cine_win.setup_progress_timer();
        cine_win.setup_close_request();

        if settings.borrow().save_session {
            let saved = load_playlist_session();
            if !saved.is_empty() {
                let gfiles: Vec<gio::File> = saved.into_iter().map(gio::File::for_path).collect();
                cine_win.open_files(&gfiles, true);
            }
        }

        cine_win
    }

    pub fn present(&self) {
        self.window.present();
        glib::timeout_add_local_once(Duration::from_millis(80), || {
            center_active_cine_window();
        });
    }

    pub fn show_toast(&self, msg: &str) {
        let toast = adw::Toast::builder().title(msg).timeout(2).build();
        self.toast_overlay.add_toast(toast);
    }

    pub fn update_quality_menu(&self, max_height: i32) {
        let quality_menu = gio::Menu::new();
        quality_menu.append(
            Some("Automático (Calidad óptima del video)"),
            Some("win.set-quality('best')"),
        );

        if max_height >= 2100 {
            quality_menu.append(Some("4K UHD (2160p)"), Some("win.set-quality('2160')"));
        }
        if max_height >= 1400 {
            quality_menu.append(Some("1440p QHD (2K)"), Some("win.set-quality('1440')"));
        }
        if max_height >= 1000 {
            quality_menu.append(Some("1080p FHD"), Some("win.set-quality('1080')"));
        }
        if max_height >= 700 {
            quality_menu.append(Some("720p HD"), Some("win.set-quality('720')"));
        }
        if max_height >= 440 {
            quality_menu.append(Some("480p SD"), Some("win.set-quality('480')"));
        }
        if max_height >= 320 {
            quality_menu.append(Some("360p"), Some("win.set-quality('360')"));
        }

        self.quality_menu_btn.set_menu_model(Some(&quality_menu));
    }

    pub fn set_controls_visible(&self, visible: bool) {
        if self.start_page.is_visible() {
            self.headerbar.set_visible(true);
            self.controls_box.set_visible(false);
            return;
        }

        if self.is_locked.get() {
            self.headerbar.set_visible(false);
            self.controls_box.set_visible(false);
            return;
        }

        self.headerbar.set_visible(visible);
        self.controls_box.set_visible(visible);

        if !visible {
            self.cancel_autohide_timer();
        }
    }

    pub fn lock_controls(&self) {
        if self.start_page.is_visible() {
            return;
        }
        self.is_locked.set(true);
        self.headerbar.set_visible(false);
        self.controls_box.set_visible(false);
        self.cancel_autohide_timer();
        self.reset_floating_lock_timer();
    }

    pub fn unlock_controls(&self) {
        self.is_locked.set(false);
        self.cancel_floating_lock_timer();
        self.floating_lock_btn.set_visible(false);
        self.headerbar.set_visible(true);
        self.controls_box.set_visible(true);
        self.reset_autohide_timer();
    }

    pub fn toggle_lock(&self) {
        if self.is_locked.get() {
            self.unlock_controls();
        } else {
            self.lock_controls();
        }
    }

    pub fn cancel_floating_lock_timer(&self) {
        if let Some(source_id) = self.lock_timer_id.borrow_mut().take() {
            source_id.remove();
        }
    }

    pub fn reset_floating_lock_timer(&self) {
        self.cancel_floating_lock_timer();
        if !self.is_locked.get() {
            return;
        }

        self.floating_lock_btn.set_visible(true);

        let this = self.clone();
        let source_id = glib::timeout_add_local(Duration::from_secs(5), move || {
            if this.is_locked.get() {
                this.floating_lock_btn.set_visible(false);
            }
            *this.lock_timer_id.borrow_mut() = None;
            glib::ControlFlow::Break
        });
        *self.lock_timer_id.borrow_mut() = Some(source_id);
    }

    pub fn cancel_autohide_timer(&self) {
        if let Some(source_id) = self.autohide_timer_id.borrow_mut().take() {
            source_id.remove();
        }
    }

    pub fn reset_autohide_timer(&self) {
        self.cancel_autohide_timer();
        if self.start_page.is_visible() {
            return;
        }

        let this = self.clone();
        let source_id = glib::timeout_add_local(Duration::from_secs(3), move || {
            if this.is_playing.get() && this.window.visible_dialog().is_none() {
                this.set_controls_visible(false);
            }
            *this.autohide_timer_id.borrow_mut() = None;
            glib::ControlFlow::Break
        });
        *self.autohide_timer_id.borrow_mut() = Some(source_id);
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
        if paused {
            self.set_controls_visible(true);
            self.cancel_autohide_timer();
        } else {
            self.reset_autohide_timer();
        }
    }

    pub fn play_prev(&self) {
        let idx = self.current_index.get();
        if idx > 0 {
            self.play_index(idx - 1);
        }
    }

    pub fn play_next(&self) {
        let len = self.playlist_items.borrow().len();
        if len == 0 {
            return;
        }
        let cur = self.current_index.get();
        if self.shuffle_enabled.get() && len > 1 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as usize)
                .unwrap_or(0);
            let mut next_idx = now % len;
            if next_idx == cur {
                next_idx = (cur + 1) % len;
            }
            self.play_index(next_idx);
        } else if cur + 1 < len {
            self.play_index(cur + 1);
        } else if self.loop_mode.get() == 2 {
            self.play_index(0);
        }
    }

    pub fn play_index(&self, new_idx: usize) {
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
        self.max_detected_height.set(0);
        self.update_quality_menu(0);
        self.quality_menu_btn.set_label("Auto");
        self.video_area.load_file(&target_path, "replace");
        self.window.set_title(Some(&target_title));
        self.title_widget.set_title(&target_title);
        self.is_locked.set(false);
        self.cancel_floating_lock_timer();
        self.floating_lock_btn.set_visible(false);
        self.set_controls_visible(true);
        self.reset_autohide_timer();
    }

    pub fn remove_playlist_index(&self, idx: usize) {
        let mut items = self.playlist_items.borrow_mut();
        if idx >= items.len() {
            return;
        }
        items.remove(idx);
        let len = items.len();
        let cur = self.current_index.get();
        if len == 0 {
            drop(items);
            self.clear_playlist();
            return;
        }

        let new_cur = if idx < cur {
            cur - 1
        } else if idx == cur {
            if idx >= len { len - 1 } else { idx }
        } else {
            cur
        };
        self.current_index.set(new_cur);

        if idx == cur {
            for (i, item) in items.iter_mut().enumerate() {
                item.playing = i == new_cur;
            }
            let target_path = items[new_cur].path.to_string_lossy().into_owned();
            let target_title = items[new_cur].title.clone();
            drop(items);

            self.last_video_dims.set((0, 0));
            self.video_area.load_file(&target_path, "replace");
            self.window.set_title(Some(&target_title));
            self.title_widget.set_title(&target_title);
        } else {
            drop(items);
        }

        if self.settings.borrow().save_session {
            save_playlist_session(&self.playlist_items.borrow());
        }
    }

    pub fn clear_playlist(&self) {
        self.playlist_items.borrow_mut().clear();
        self.current_index.set(0);
        self.max_detected_height.set(0);
        self.update_quality_menu(0);
        self.quality_menu_btn.set_label("Auto");
        self.video_area.load_file("", "replace");
        self.window.set_title(Some("Cine (Rust)"));
        self.title_widget.set_visible(false);
        self.controls_box.set_visible(false);
        self.headerbar.set_visible(true);
        self.cancel_autohide_timer();
        self.is_locked.set(false);
        self.cancel_floating_lock_timer();
        self.floating_lock_btn.set_visible(false);
        self.start_page.set_visible(true);
        self.is_playing.set(false);
        self.sync_inhibit();

        if self.settings.borrow().save_session {
            save_playlist_session(&[]);
        }
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
        glib::timeout_add_local_once(Duration::from_millis(150), || {
            center_active_cine_window();
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
                Some("Reproduciendo contenido"),
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
                let is_network_stream = this
                    .playlist_items
                    .borrow()
                    .get(this.current_index.get())
                    .map(|item| {
                        let path = item.path.to_string_lossy();
                        path.starts_with("http://") || path.starts_with("https://")
                    })
                    .unwrap_or(false);

                if is_network_stream && h > this.max_detected_height.get() {
                    this.max_detected_height.set(h);
                    this.update_quality_menu(h);
                } else if !is_network_stream && this.max_detected_height.replace(0) != 0 {
                    // Los archivos locales solo tienen la calidad original; no ofrecer
                    // límites de resolución que no corresponden a variantes disponibles.
                    this.update_quality_menu(0);
                }
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

                if !paused && dur > 2.0 && pos >= dur - 0.4 {
                    if this.loop_mode.get() == 1 {
                        this.video_area.seek_absolute(0.0);
                    } else {
                        this.play_next();
                    }
                }
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
            // Bloquear todas las teclas del reproductor; solo 'L' vuelve a habilitar controles.
            if this.is_locked.get() {
                if matches!(key, gdk::Key::l | gdk::Key::L) {
                    this.unlock_controls();
                }
                return glib::Propagation::Stop;
            }

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
                gdk::Key::l | gdk::Key::L => {
                    this.toggle_lock();
                    glib::Propagation::Stop
                }
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
                gdk::Key::a | gdk::Key::A => {
                    this.video_area.command(&["cycle", "audio"]);
                    glib::Propagation::Stop
                }
                gdk::Key::s | gdk::Key::S => {
                    this.video_area.command(&["cycle", "sub"]);
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

    fn setup_mouse_gestures(&self, overlay: &gtk::Overlay) {
        // Gesto de doble clic en video: alternar pantalla completa
        let click_gesture = gtk::GestureClick::new();
        click_gesture.set_button(gdk::BUTTON_PRIMARY);

        let this = self.clone();
        click_gesture.connect_pressed(move |_gesture, n_press, _x, _y| {
            if this.start_page.is_visible() || this.is_locked.get() {
                return;
            }

            if n_press == 2 {
                if this.window.is_fullscreen() {
                    this.window.unfullscreen();
                } else {
                    this.window.fullscreen();
                }
            }
        });
        overlay.add_controller(click_gesture);

        // Movimiento de ratón: muestra los controles (o el candado flotante si está bloqueado)
        let motion_ctrl = gtk::EventControllerMotion::new();
        let this = self.clone();

        motion_ctrl.connect_motion(move |_, _x, _y| {
            if this.start_page.is_visible() {
                return;
            }

            // Si está bloqueado con el candado, los controles no aparecen
            // y solo se renueva la visibilidad temporal del candado durante 5 segundos
            if this.is_locked.get() {
                this.reset_floating_lock_timer();
                return;
            }

            if !this.controls_box.is_visible() || !this.headerbar.is_visible() {
                this.set_controls_visible(true);
            }
            this.reset_autohide_timer();
        });
        overlay.add_controller(motion_ctrl);
    }

    fn setup_close_request(&self) {
        let this = self.clone();
        self.window.connect_close_request(move |_| {
            if this.settings.borrow().save_session {
                save_playlist_session(&this.playlist_items.borrow());
            }
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
        let this = self.clone();
        action_history.connect_activate(move |_, _| {
            let th = this.clone();
            present_history_dialog(&this.window, move |path| {
                if path.starts_with("http://") || path.starts_with("https://") {
                    th.open_url(path, true);
                } else {
                    let file = gio::File::for_path(path);
                    th.open_files(&[file], true);
                }
            });
        });
        self.window.add_action(&action_history);

        // win.open-playlist-dialog (Ctrl+P)
        let action_playlist = gio::SimpleAction::new("open-playlist-dialog", None);
        let this = self.clone();
        action_playlist.connect_activate(move |_, _| {
            let th_sel = this.clone();
            let th_rem = this.clone();
            let th_clr = this.clone();
            let th_add_files = this.clone();
            let th_add_url = this.clone();
            let items = this.playlist_items.borrow().clone();
            present_playlist_dialog(
                &this.window,
                &items,
                move |idx| th_sel.play_index(idx),
                move |idx| th_rem.remove_playlist_index(idx),
                move || th_clr.clear_playlist(),
                move || th_add_files.open_file_dialog(false),
                move || th_add_url.open_url_dialog(false),
            );
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
            let (fmt, toast_msg) = match tier {
                "2160" => (
                    "bestvideo[height<=2160]+bestaudio[language~=es]/bestvideo[height<=2160]+bestaudio/best[height<=2160]/best",
                    "Calidad fijada a 4K UHD",
                ),
                "1440" => (
                    "bestvideo[height<=1440]+bestaudio[language~=es]/bestvideo[height<=1440]+bestaudio/best[height<=1440]/best",
                    "Calidad fijada a 1440p QHD",
                ),
                "1080" => (
                    "bestvideo[height<=1080]+bestaudio[language~=es]/bestvideo[height<=1080]+bestaudio/best[height<=1080]/best",
                    "Calidad fijada a 1080p FHD",
                ),
                "720" => (
                    "bestvideo[height<=720]+bestaudio[language~=es]/bestvideo[height<=720]+bestaudio/best[height<=720]/best",
                    "Calidad fijada a 720p HD",
                ),
                "480" => (
                    "bestvideo[height<=480]+bestaudio[language~=es]/bestvideo[height<=480]+bestaudio/best[height<=480]/best",
                    "Calidad fijada a 480p SD",
                ),
                "360" => (
                    "bestvideo[height<=360]+bestaudio[language~=es]/bestvideo[height<=360]+bestaudio/best[height<=360]/best",
                    "Calidad fijada a 360p",
                ),
                _ => (
                    "bestvideo+bestaudio[language~=es]/bestvideo+bestaudio/best",
                    "Calidad en Automático",
                ),
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
                this.show_toast(toast_msg);
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

    #[allow(deprecated)]
    fn open_track_file_dialog<FToast>(
        parent: &adw::ApplicationWindow,
        video_area: &VideoGLArea,
        mpv_cmd: &'static str,
        title: &str,
        toast_fn: FToast,
    ) where
        FToast: Fn(&str) + 'static,
    {
        let dialog = gtk::FileChooserDialog::new(
            Some(title),
            Some(parent),
            gtk::FileChooserAction::Open,
            &[
                ("Cancelar", gtk::ResponseType::Cancel),
                ("Añadir", gtk::ResponseType::Accept),
            ],
        );
        dialog.set_modal(true);

        let va = video_area.clone();
        dialog.connect_response(move |dlg, response| {
            if response == gtk::ResponseType::Accept {
                if let Some(file) = dlg.file() {
                    if let Some(path) = file.path() {
                        va.command(&[mpv_cmd, &path.to_string_lossy(), "select"]);
                        toast_fn("Pista añadida correctamente");
                    }
                }
            }
            dlg.destroy();
        });
        dialog.present();
    }

    pub fn open_url(&self, url: &str, clear_existing: bool) {
        eprintln!("[CineRust] Abriendo URL: {url}");
        record_history_entry(url, url);
        self.last_video_dims.set((0, 0));
        self.max_detected_height.set(0);
        self.update_quality_menu(0);
        self.quality_menu_btn.set_label("Auto");
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
        if self.settings.borrow().save_session {
            save_playlist_session(&items);
        }
        drop(items);

        self.window.set_title(Some(url));
        self.title_widget.set_title(url);
        self.title_widget.set_visible(true);
        self.start_page.set_visible(false);
        self.is_locked.set(false);
        self.cancel_floating_lock_timer();
        self.floating_lock_btn.set_visible(false);
        self.set_controls_visible(true);
        self.reset_autohide_timer();
        self.is_playing.set(true);
        self.play_pause_btn
            .set_icon_name("cine-playback-pause-symbolic");
        self.sync_inhibit();
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
            record_history_entry(&path.to_string_lossy(), &title);
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
        if self.settings.borrow().save_session {
            save_playlist_session(&items);
        }
        drop(items);

        if clear_existing || start_empty {
            self.last_video_dims.set((0, 0));
            self.max_detected_height.set(0);
            self.update_quality_menu(0);
            self.quality_menu_btn.set_label("Auto");
            let display_name = first_path
                .file_name()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Video".to_string());
            self.window.set_title(Some(&display_name));
            self.title_widget.set_title(&display_name);
        }

        self.title_widget.set_visible(true);
        self.start_page.set_visible(false);
        self.is_locked.set(false);
        self.cancel_floating_lock_timer();
        self.floating_lock_btn.set_visible(false);
        self.set_controls_visible(true);
        self.reset_autohide_timer();
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
