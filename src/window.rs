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
mod actions;
mod builder;
mod controls;
mod dialogs;
mod input;
mod playback;
mod surface;
mod ui;

use crate::utils::{
    center_cine_window, center_x11_surface, collect_media_files, format_time, resolution_badge,
};

#[derive(Clone)]
pub struct CineWindow {
    pub window: adw::ApplicationWindow,
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
    initial_window_size: Rc<Cell<Option<(i32, i32)>>>,
    manual_video_size_active: Rc<Cell<bool>>,
    resize_key_down: Rc<Cell<bool>>,
    resize_generation: Rc<Cell<u64>>,
    resize_request_generation: Rc<Cell<u64>>,
    pub options: Rc<RefCell<PlaybackOptions>>,
    zoom_indicator: gtk::Label,
    zoom_indicator_timer: Rc<RefCell<Option<glib::SourceId>>>,
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
    pub fn present(&self) {
        self.window.present();
        for delay in [250, 800] {
            let this = self.clone();
            glib::timeout_add_local_once(Duration::from_millis(delay), move || {
                this.center_on_current_monitor();
            });
        }
    }

    pub(super) fn center_on_current_monitor(&self) {
        if let (Some(display), Some(surface)) = (gdk::Display::default(), self.window.surface()) {
            if center_x11_surface(&display, &surface) {
                return;
            }
        }
        center_cine_window();
    }

    pub fn show_toast(&self, _msg: &str) {
        // Los avisos emergentes del reproductor están desactivados.
    }

    pub fn show_zoom_indicator(&self, zoom: f64) {
        update_zoom_indicator(&self.zoom_indicator, &self.zoom_indicator_timer, zoom);
    }
}

pub(crate) fn update_zoom_indicator(
    label: &gtk::Label,
    timer_id: &Rc<RefCell<Option<glib::SourceId>>>,
    zoom: f64,
) {
    let percent = 100.0 * 2.0_f64.powf(zoom);
    label.set_label(&format!("{percent:.0}%"));
    label.set_visible(true);

    if let Some(timer) = timer_id.borrow_mut().take() {
        timer.remove();
    }
    let weak_label = label.downgrade();
    let timer_id_for_callback = timer_id.clone();
    let timer = glib::timeout_add_local_once(Duration::from_millis(1200), move || {
        if let Some(label) = weak_label.upgrade() {
            label.set_visible(false);
        }
        timer_id_for_callback.borrow_mut().take();
    });
    *timer_id.borrow_mut() = Some(timer);
}
