// preferences.rs
//
// Gestión de preferencias de usuario y diálogo de configuración (Preferences).
// Persiste la configuración en `~/.config/cine/settings.conf` mediante `glib::KeyFile`.
// Equivalente modular a `preferences.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;
use gtk::glib;

use crate::utils::config_dir;

/// Estado de configuración de la aplicación.
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
            auto_resize_window: true, // Ajustar la ventana a la relación de aspecto del video
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

impl AppSettings {
    /// Carga las preferencias guardadas desde disco o genera los valores por defecto.
    pub fn load_from_disk() -> Self {
        let mut settings = Self::default();
        let path = config_dir().join("settings.conf");
        let keyfile = glib::KeyFile::new();
        if keyfile.load_from_file(&path, glib::KeyFileFlags::NONE).is_ok() {
            if let Ok(v) = keyfile.boolean("General", "open_new_windows") {
                settings.open_new_windows = v;
            }
            if let Ok(v) = keyfile.boolean("General", "auto_resize_window") {
                settings.auto_resize_window = v;
            }
            if let Ok(v) = keyfile.boolean("General", "hwdec") {
                settings.hwdec = v;
            }
            if let Ok(v) = keyfile.boolean("General", "graphics_offload") {
                settings.graphics_offload = v;
            }
            if let Ok(v) = keyfile.boolean("General", "save_video_position") {
                settings.save_video_position = v;
            }
            if let Ok(v) = keyfile.boolean("General", "save_session") {
                settings.save_session = v;
            }
            if let Ok(v) = keyfile.integer("General", "volume") {
                settings.volume = v;
            }
            if let Ok(v) = keyfile.string("Subtitles", "subtitle_color") {
                settings.subtitle_color = v.to_string();
            }
            if let Ok(v) = keyfile.string("Subtitles", "subtitle_font") {
                settings.subtitle_font = v.to_string();
            }
        }
        settings
    }

    /// Guarda las preferencias en disco.
    pub fn save_to_disk(&self) {
        let dir = config_dir();
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("settings.conf");

        let keyfile = glib::KeyFile::new();
        keyfile.set_boolean("General", "open_new_windows", self.open_new_windows);
        keyfile.set_boolean("General", "auto_resize_window", self.auto_resize_window);
        keyfile.set_boolean("General", "hwdec", self.hwdec);
        keyfile.set_boolean("General", "graphics_offload", self.graphics_offload);
        keyfile.set_boolean("General", "save_video_position", self.save_video_position);
        keyfile.set_boolean("General", "save_session", self.save_session);
        keyfile.set_integer("General", "volume", self.volume);
        keyfile.set_string("Subtitles", "subtitle_color", &self.subtitle_color);
        keyfile.set_string("Subtitles", "subtitle_font", &self.subtitle_font);

        let _ = keyfile.save_to_file(path);
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
            settings.borrow().save_to_disk();
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
            settings.borrow().save_to_disk();
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
            settings.borrow().save_to_disk();
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
            settings.borrow().save_to_disk();
        });
    }

    let save_session_row = adw::SwitchRow::builder()
        .title("Restaurar última lista de reproducción al iniciar")
        .active(settings.borrow().save_session)
        .build();
    {
        let settings = settings.clone();
        save_session_row.connect_active_notify(move |row| {
            settings.borrow_mut().save_session = row.is_active();
            settings.borrow().save_to_disk();
        });
    }

    group_general.add(&hwdec_row);
    group_general.add(&auto_resize_row);
    group_general.add(&open_new_row);
    group_general.add(&save_pos_row);
    group_general.add(&save_session_row);
    page.add(&group_general);

    dialog.add(&page);
    dialog.present(Some(parent));
}
