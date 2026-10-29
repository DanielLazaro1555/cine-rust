// preferences.rs
//
// Gestión de preferencias de usuario y diálogo de configuración (Preferences).
// Equivalente modular a `preferences.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;

/// Estado de configuración en memoria (sincronizable con GSettings / config).
#[derive(Clone, Debug)]
pub struct AppSettings {
    pub open_new_windows: bool,
    pub auto_resize_window: bool,
    pub hwdec: bool,
    pub graphics_offload: bool,
    pub normalize_volume: bool,
    pub save_video_position: bool,
    pub save_session: bool,
    pub thumbnail_preview: bool,
    pub show_remaining: bool,
    pub volume: i32,
    pub subtitle_scale: f64,
    pub subtitle_color: String,
    pub subtitle_font: String,
    pub subtitle_languages: String,
    pub audio_languages: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            open_new_windows: true,
            auto_resize_window: true, // Ajustar la ventana al tamaño/proporción del video
            hwdec: true,              // Activo por defecto para usar GPU (NVDEC / VAAPI / Vulkan)
            graphics_offload: false,
            normalize_volume: false,
            save_video_position: false,
            save_session: false,
            thumbnail_preview: false,
            show_remaining: false,
            volume: 100,
            subtitle_scale: 1.0,
            subtitle_color: "#ebebeb".to_string(),
            subtitle_font: "Adwaita Sans SemiBold".to_string(),
            subtitle_languages: String::new(),
            audio_languages: String::new(),
        }
    }
}

pub type SharedSettings = Rc<RefCell<AppSettings>>;

/// Abre el diálogo de Preferencias de Cine.
pub fn present_preferences_dialog(parent: &impl IsA<gtk::Widget>, settings: SharedSettings) {
    let dialog = adw::PreferencesDialog::builder()
        .title("Preferencias")
        .build();

    let page = adw::PreferencesPage::new();

    // Grupo: Comportamiento y Rendimiento (GPU)
    let group_general = adw::PreferencesGroup::builder()
        .title("General y Rendimiento")
        .build();

    let hwdec_row = adw::SwitchRow::builder()
        .title("Decodificación por hardware (GPU)")
        .subtitle("Usa NVDEC / VA-API / Vulkan para decodificar video en la tarjeta gráfica")
        .active(settings.borrow().hwdec)
        .build();
    {
        let settings = settings.clone();
        hwdec_row.connect_active_notify(move |row| {
            settings.borrow_mut().hwdec = row.is_active();
        });
    }

    let open_new_row = adw::SwitchRow::builder()
        .title("Abrir archivos en ventana nueva")
        .active(settings.borrow().open_new_windows)
        .build();
    {
        let settings = settings.clone();
        open_new_row.connect_active_notify(move |row| {
            settings.borrow_mut().open_new_windows = row.is_active();
        });
    }

    let save_pos_row = adw::SwitchRow::builder()
        .title("Guardar posición del video al cerrar")
        .active(settings.borrow().save_video_position)
        .build();
    {
        let settings = settings.clone();
        save_pos_row.connect_active_notify(move |row| {
            settings.borrow_mut().save_video_position = row.is_active();
        });
    }

    let auto_resize_row = adw::SwitchRow::builder()
        .title("Ajustar ventana al tamaño del video")
        .subtitle("Redimensiona automáticamente la ventana según la resolución y proporción del video")
        .active(settings.borrow().auto_resize_window)
        .build();
    {
        let settings = settings.clone();
        auto_resize_row.connect_active_notify(move |row| {
            settings.borrow_mut().auto_resize_window = row.is_active();
        });
    }

    group_general.add(&hwdec_row);
    group_general.add(&auto_resize_row);
    group_general.add(&open_new_row);
    group_general.add(&save_pos_row);
    page.add(&group_general);

    dialog.add(&page);
    dialog.present(Some(parent));
}
