// Construcción y conexión inicial de la interfaz de CineWindow.
use super::*;

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
            "../../resources/style.css"
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
        if settings.borrow().normalize_volume {
            video_area.set_normalize_volume(true);
        }
        video_area.set_visible(true);
        video_overlay.set_child(Some(video_area.widget()));

        let (title_widget, headerbar) = surface::build_headerbar();

        let start_page = surface::build_start_page();

        let ui::PlaybackControlsUi {
            controls_box,
            prev_btn,
            play_pause_btn,
            next_btn,
            vol_scale,
            quality_menu_btn,
            progress_adj,
            progress_scale,
            time_elapsed_label,
            time_total_label,
            options,
            shuffle_btn,
            repeat_btn,
            lock_btn,
            floating_lock_btn,
            fullscreen_btn,
        } = ui::build_playback_controls(
            &video_overlay,
            &toast_overlay,
            &window,
            &video_area,
            &headerbar,
            &start_page,
        );

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

}
