// CineWindow responsibility module.
use super::*;

impl CineWindow {
    #[allow(deprecated)]
    pub(super) fn open_file_dialog(&self, clear_existing: bool) {
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
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Video y audio"));
        for ext in crate::utils::VIDEO_EXTENSIONS {
            filter.add_suffix(ext);
        }
        dialog.add_filter(&filter);

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
    pub(super) fn open_folder_dialog(&self, clear_existing: bool) {
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

    pub(super) fn open_url_dialog(&self, clear_existing: bool) {
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
    pub(super) fn add_external_track_dialog(&self, mpv_cmd: &'static str, title: &str) {
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
    pub(super) fn open_track_file_dialog<FToast>(
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
                media_paths.extend(collect_media_files(&path));
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
    }}
