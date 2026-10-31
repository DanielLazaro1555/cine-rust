// CineWindow responsibility module.
use super::*;
use crate::utils::{save_window_position, x11_surface_position};

impl CineWindow {
    pub(super) fn rotate_video(&self) {
        let rotation = (self.video_rotation.get() + 90) % 360;
        self.video_rotation.set(rotation);
        self.video_area.set_video_rotate(rotation);
    }

    pub(super) fn setup_close_request(&self) {
        let this = self.clone();
        self.window.connect_close_request(move |_| {
            if let (Some(display), Some(surface)) = (gdk::Display::default(), this.window.surface()) {
                if let Some((x, y)) = x11_surface_position(&display, &surface) {
                    save_window_position(x, y);
                }
            }
            if this.settings.borrow().save_session {
                save_playlist_session(&this.playlist_items.borrow());
            }
            this.is_playing.set(false);
            this.sync_inhibit();
            this.video_area.shutdown();
            glib::Propagation::Proceed
        });
    }

    pub(super) fn setup_actions(&self) {
        // win.quit (tecla 'q' o Ctrl+W)
        let action_quit = gio::SimpleAction::new("quit", None);
        let win_weak = self.window.downgrade();
        action_quit.connect_activate(move |_, _| {
            if let Some(win) = win_weak.upgrade() {
                win.close();
            }
        });
        self.window.add_action(&action_quit);

        // Aplica la preferencia global de normalización a esta ventana.
        let action_normalize = gio::SimpleAction::new("apply-normalize-volume", None);
        let this = self.clone();
        action_normalize.connect_activate(move |_, _| {
            this.video_area
                .set_normalize_volume(this.settings.borrow().normalize_volume);
        });
        self.window.add_action(&action_normalize);

        // win.rotate-video (Ctrl+Shift+G)
        let action_rotate = gio::SimpleAction::new("rotate-video", None);
        let this = self.clone();
        action_rotate.connect_activate(move |_, _| this.rotate_video());
        self.window.add_action(&action_rotate);

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

}
