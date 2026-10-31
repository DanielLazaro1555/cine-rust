// Construcción de la barra OSD y sus controles.
use super::*;

pub(super) struct PlaybackControlsUi {
    pub(super) controls_box: gtk::Box ,
    pub(super) prev_btn: gtk::Button ,
    pub(super) play_pause_btn: gtk::Button ,
    pub(super) next_btn: gtk::Button ,
    pub(super) vol_scale: gtk::Scale ,
    pub(super) quality_menu_btn: gtk::MenuButton,
    pub(super) progress_adj: gtk::Adjustment ,
    pub(super) progress_scale: gtk::Scale ,
    pub(super) time_elapsed_label: gtk::Label ,
    pub(super) time_total_label: gtk::Label ,
    pub(super) options: Rc<RefCell<PlaybackOptions>> ,
    pub(super) shuffle_btn: gtk::Button ,
    pub(super) repeat_btn: gtk::Button ,
    pub(super) lock_btn: gtk::Button ,
    pub(super) floating_lock_btn: gtk::Button ,
    pub(super) fullscreen_btn: gtk::Button ,
}

pub(super) fn build_playback_controls(
    video_overlay: &gtk::Overlay,
    toast_overlay: &adw::ToastOverlay,
    window: &adw::ApplicationWindow,
    video_area: &VideoGLArea,
    headerbar: &adw::HeaderBar,
    start_page: &adw::StatusPage,
    zoom_indicator: &gtk::Label,
    zoom_indicator_timer: &Rc<RefCell<Option<glib::SourceId>>>,
) -> PlaybackControlsUi {
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
                    CineWindow::open_track_file_dialog(&win, &va, "audio-add", "Añadir pista de audio externa", t_notify.clone());
                }
            }
        };

        let add_sub_cb = {
            let win_weak = window.downgrade();
            let va = video_area.clone();
            let t_notify = toast_notifier.clone();
            move || {
                if let Some(win) = win_weak.upgrade() {
                    CineWindow::open_track_file_dialog(&win, &va, "sub-add", "Añadir archivo de subtítulos", t_notify.clone());
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
        let options_menu_btn = create_options_menu_button(
            video_area.clone(),
            options.clone(),
            zoom_indicator.clone(),
            zoom_indicator_timer.clone(),
        );

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
        ui_box.append(headerbar);
        let spacer = gtk::Separator::new(gtk::Orientation::Horizontal);
        spacer.set_vexpand(true);
        spacer.set_opacity(0.0);
        spacer.set_can_target(false);
        ui_box.append(&spacer);
        ui_box.append(&controls_box);

        video_overlay.add_overlay(&ui_box);
        video_overlay.add_overlay(&floating_lock_btn);
        video_overlay.add_overlay(start_page);

        PlaybackControlsUi {
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
        }
}
