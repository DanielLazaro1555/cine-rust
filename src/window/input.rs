// CineWindow responsibility module.
use super::*;

impl CineWindow {
    pub(super) fn setup_keyboard_and_drop(&self, overlay: &gtk::Overlay) {
        let key_ctrl = gtk::EventControllerKey::new();
        // Fase Capture para interceptar Espacio/flechas antes de que botones o sliders los consuman
        key_ctrl.set_propagation_phase(gtk::PropagationPhase::Capture);
        let this = self.clone();
        key_ctrl.connect_key_pressed(move |_, key, _, _| {
            if matches!(key, gdk::Key::r | gdk::Key::R) && this.resize_key_down.replace(true) {
                return glib::Propagation::Stop;
            }

            // Bloquear las teclas del reproductor; L desbloquea y R restaura
            // el tamaño original antes de desbloquear si estaba ampliado.
            if this.is_locked.get() {
                match key {
                    gdk::Key::l | gdk::Key::L => this.unlock_controls(),
                    gdk::Key::r | gdk::Key::R => {
                        this.request_video_window_size_toggle();
                        this.unlock_controls();
                    }
                    _ => {}
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
                gdk::Key::r | gdk::Key::R => {
                    this.request_video_window_size_toggle();
                    glib::Propagation::Stop
                }
                gdk::Key::_0 => {
                    this.options.borrow_mut().video_zoom = 0.0;
                    this.video_area.reset_video_view();
                    this.show_zoom_indicator(0.0);
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
        let this = self.clone();
        key_ctrl.connect_key_released(move |_, key, _, _| {
            if matches!(key, gdk::Key::r | gdk::Key::R) {
                this.resize_key_down.set(false);
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

    pub(super) fn setup_mouse_gestures(&self, overlay: &gtk::Overlay) {
        let this = self.clone();
        crate::gestures::setup_video_gestures(
            overlay,
            &[
                self.controls_box.upcast_ref::<gtk::Widget>(),
                self.headerbar.upcast_ref::<gtk::Widget>(),
                self.floating_lock_btn.upcast_ref::<gtk::Widget>(),
            ],
            &self.window,
            &self.start_page,
            &self.is_locked,
            {
                let overlay = overlay.clone();
                move |direction| {
                    if this.start_page.is_visible() || this.is_locked.get() {
                        return glib::Propagation::Proceed;
                    }

                    let mut options = this.options.borrow_mut();
                    let next_zoom = (options.video_zoom - direction * 0.1).clamp(0.0, 3.0);
                    options.video_zoom = next_zoom;
                    drop(options);
                    this.video_area.set_video_zoom(next_zoom);
                    this.show_zoom_indicator(next_zoom);
                    overlay.set_cursor_from_name(Some(if next_zoom > 0.0 {
                        "grab"
                    } else {
                        "default"
                    }));
                    glib::Propagation::Stop
                }
            },
            {
                let this = self.clone();
                let overlay = overlay.clone();
                let start_pan = Rc::new(Cell::new((0.0_f64, 0.0_f64)));
                let pending_pan = Rc::new(Cell::new((0.0_f64, 0.0_f64)));
                let pan_update_scheduled = Rc::new(Cell::new(false));
                move |dx, dy, begin| {
                    let zoom = this.options.borrow().video_zoom;
                    if zoom <= 0.0 || this.start_page.is_visible() || this.is_locked.get() {
                        return false;
                    }
                    if begin {
                        start_pan.set(this.video_area.video_pan());
                        overlay.set_cursor_from_name(Some("grabbing"));
                        return true;
                    }
                    let (pan_x, pan_y) = start_pan.get();
                    let (x, y) = this
                        .video_area
                        .video_pan_for_drag(pan_x, pan_y, dx, dy, zoom);
                    pending_pan.set((x, y));
                    if !pan_update_scheduled.replace(true) {
                        let this = this.clone();
                        let pending_pan = pending_pan.clone();
                        let pan_update_scheduled = pan_update_scheduled.clone();
                        glib::timeout_add_local_once(Duration::from_millis(16), move || {
                            pan_update_scheduled.set(false);
                            let (x, y) = pending_pan.get();
                            this.video_area.set_video_pan(x, y);
                        });
                    }
                    overlay.set_cursor_from_name(Some("grabbing"));
                    true
                }
            },
            {
                let this = self.clone();
                move || this.reset_floating_lock_timer()
            },
            {
                let this = self.clone();
                let overlay = overlay.clone();
                move || {
                    overlay.set_cursor_from_name(Some(if this.options.borrow().video_zoom > 0.0 {
                        "grab"
                    } else {
                        "default"
                    }));
                    if !this.controls_box.is_visible() || !this.headerbar.is_visible() {
                        this.set_controls_visible(true);
                    }
                    this.reset_autohide_timer();
                }
            },
        );
    }

}
