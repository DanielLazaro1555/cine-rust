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
    calculate_window_size, center_active_cine_window, collect_media_files, format_time,
    install_cine_window_added_centering, resolution_badge,
};

#[derive(Clone)]
pub struct CineWindow {
    pub window: adw::ApplicationWindow,
    toast_overlay: adw::ToastOverlay,
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
    pub options: Rc<RefCell<PlaybackOptions>>,
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
        install_cine_window_added_centering();
        self.window.present();
        glib::timeout_add_local_once(Duration::from_millis(80), || {
            center_active_cine_window();
        });
    }

    pub fn show_toast(&self, msg: &str) {
        let toast = adw::Toast::builder().title(msg).timeout(2).build();
        self.toast_overlay.add_toast(toast);
    }


}
