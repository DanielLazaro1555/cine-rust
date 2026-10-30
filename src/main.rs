// main.rs
//
// Punto de entrada de Cine Rust.
// Registra los recursos embebidos (`cine.gresource`) y delega en `CineApplication`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod application;
pub mod gestures;
pub mod history;
pub mod mpris;
pub mod mpv_engine;
pub mod mpv_gl_area;
pub mod options;
pub mod playlist;
pub mod preferences;
pub mod save_session;
pub mod shortcuts;
pub mod tracks;
pub mod utils;
pub mod window;

use gtk::{gio, glib};

use crate::application::CineApplication;

pub const APP_ID: &str = "io.github.diegopvlk.Cine";
pub const ICON_NAME: &str = "io.github.diegopvlk.Cine";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

static GRESOURCE_BYTES: &[u8] = include_bytes!("../resources/cine.gresource");

fn main() -> glib::ExitCode {
    // SAFETY: se configura al arranque en el hilo principal antes de iniciar GTK/hilos.
    unsafe {
        std::env::set_var("GSK_RENDERER", "gl");
        // Al ser una aplicación nativa fuera de Flatpak, usar el selector nativo de GTK4
        // directamente en proceso sin pasar por xdg-desktop-portal.
        std::env::set_var("GTK_USE_PORTAL", "0");
        std::env::set_var(
            "PIPEWIRE_PROPS",
            "{application.icon-name=\"io.github.diegopvlk.Cine\"}",
        );
    }

    glib::set_prgname(Some("cine"));
    glib::set_application_name("Cine (Rust)");

    let resource = gio::Resource::from_data(&glib::Bytes::from_static(GRESOURCE_BYTES))
        .expect("No se pudo cargar cine.gresource embebido");
    gio::resources_register(&resource);

    let app = CineApplication::new();
    app.run()
}
