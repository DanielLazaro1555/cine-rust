// Gestures.rs
//
// Controladores de entrada del reproductor.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Instala rueda de zoom, doble clic para pantalla completa y movimiento para mostrar controles.
pub fn setup_video_gestures<F, L, M>(
    overlay: &gtk::Overlay,
    window: &adw::ApplicationWindow,
    start_page: &adw::StatusPage,
    is_locked: &Rc<Cell<bool>>,
    on_zoom_scroll: F,
    on_locked_motion: L,
    on_unlocked_motion: M,
)
where
    F: Fn(f64) -> gtk::glib::Propagation + 'static,
    L: Fn() + 'static,
    M: Fn() + 'static,
{
    let controller =
        gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    controller.connect_scroll(move |_, _, dy| {
        if dy == 0.0 {
            gtk::glib::Propagation::Proceed
        } else {
            on_zoom_scroll(dy.signum())
        }
    });
    overlay.add_controller(controller);

    let click_gesture = gtk::GestureClick::new();
    click_gesture.set_button(gtk::gdk::BUTTON_PRIMARY);
    let window = window.clone();
    let click_start_page = start_page.clone();
    let click_is_locked = is_locked.clone();
    click_gesture.connect_pressed(move |_, n_press, _, _| {
        if click_start_page.is_visible() || click_is_locked.get() || n_press != 2 {
            return;
        }
        if window.is_fullscreen() {
            window.unfullscreen();
        } else {
            window.fullscreen();
        }
    });
    overlay.add_controller(click_gesture);

    let motion_controller = gtk::EventControllerMotion::new();
    let start_page = start_page.clone();
    let is_locked = is_locked.clone();
    motion_controller.connect_motion(move |_, _, _| {
        if start_page.is_visible() {
            return;
        }
        if is_locked.get() {
            on_locked_motion();
        } else {
            on_unlocked_motion();
        }
    });
    overlay.add_controller(motion_controller);
}
