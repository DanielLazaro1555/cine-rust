// CineWindow responsibility module.
use super::*;

impl CineWindow {
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
        glib::timeout_add_local_once(Duration::from_millis(400), || {
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

    pub(super) fn setup_progress_timer(&self) {
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

}
