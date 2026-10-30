// CineWindow responsibility module.
use super::*;

impl CineWindow {
    pub(super) fn setup_keyboard_and_drop(&self, overlay: &gtk::Overlay) {
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

    pub(super) fn setup_mouse_gestures(&self, overlay: &gtk::Overlay) {
        let this = self.clone();
        crate::gestures::setup_video_gestures(
            overlay,
            &self.window,
            &self.start_page,
            &self.is_locked,
            move |direction| {
            if this.start_page.is_visible() || this.is_locked.get() {
                return glib::Propagation::Proceed;
            }

            let mut options = this.options.borrow_mut();
            let next_zoom = (options.video_zoom - direction * 0.15).clamp(-3.0, 3.0);
            options.video_zoom = next_zoom;
            drop(options);
            this.video_area.set_video_zoom(next_zoom);
            glib::Propagation::Stop
            },
            {
                let this = self.clone();
                move || this.reset_floating_lock_timer()
            },
            {
                let this = self.clone();
                move || {
                    if !this.controls_box.is_visible() || !this.headerbar.is_visible() {
                        this.set_controls_visible(true);
                    }
                    this.reset_autohide_timer();
                }
            },
        );
    }

}
