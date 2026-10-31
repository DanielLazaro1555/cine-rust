// Gestures.rs
//
// Controladores de entrada del reproductor.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Instala rueda de zoom, doble clic para pantalla completa y movimiento para mostrar controles.
pub fn setup_video_gestures<F, D, L, M>(
    overlay: &gtk::Overlay,
    control_regions: &[&gtk::Widget],
    window: &adw::ApplicationWindow,
    start_page: &adw::StatusPage,
    is_locked: &Rc<Cell<bool>>,
    on_zoom_scroll: F,
    on_pan_drag: D,
    on_locked_motion: L,
    on_unlocked_motion: M,
)
where
    F: Fn(f64) -> gtk::glib::Propagation + 'static,
    D: Fn(f64, f64, bool) -> bool + 'static,
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

    let drag = gtk::GestureDrag::new();
    drag.set_button(gtk::gdk::BUTTON_PRIMARY);
    drag.set_propagation_phase(gtk::PropagationPhase::Capture);
    let control_regions: Vec<gtk::Widget> = control_regions
        .iter()
        .map(|widget| (*widget).clone())
        .collect();
    let on_pan_drag = std::rc::Rc::new(on_pan_drag);
    let pan_active = Rc::new(Cell::new(false));
    let on_begin = on_pan_drag.clone();
    let pan_active_begin = pan_active.clone();
    let drag_begin = drag.clone();
    let overlay_for_begin = overlay.clone();
    drag.connect_drag_begin(move |_, x, y| {
        let starts_on_control = overlay_for_begin
            .pick(x, y, gtk::PickFlags::DEFAULT)
            .is_some_and(|widget| {
                control_regions
                    .iter()
                    .any(|region| widget == *region || widget.is_ancestor(region))
            });
        let active = !starts_on_control && on_begin(0.0, 0.0, true);
        pan_active_begin.set(active);
        if !active {
            drag_begin.set_state(gtk::EventSequenceState::Denied);
        } else {
            drag_begin.set_state(gtk::EventSequenceState::Claimed);
        }
    });
    let on_update = on_pan_drag.clone();
    let drag_update = drag.clone();
    drag.connect_drag_update(move |_, dx, dy| {
        if !on_update(dx, dy, false) {
            drag_update.set_state(gtk::EventSequenceState::Denied);
        } else {
            drag_update.set_state(gtk::EventSequenceState::Claimed);
        }
    });
    let overlay_for_end = overlay.clone();
    drag.connect_drag_end(move |_, _, _| {
        if pan_active.replace(false) {
            overlay_for_end.set_cursor_from_name(Some("grab"));
        }
    });
    overlay.add_controller(drag);

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
    overlay.add_controller(click_gesture.clone());

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
