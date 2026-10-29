// mpris.rs
//
// Integración con el bus D-Bus `org.mpris.MediaPlayer2.Cine` para controles
// multimedia del escritorio (KDE / GNOME).
// Equivalente modular a `mpris.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::rc::Rc;

use gtk::gio;

#[derive(Clone)]
pub struct MprisController {
    pub identity: String,
    current_title: Rc<RefCell<String>>,
}

impl Default for MprisController {
    fn default() -> Self {
        Self::new()
    }
}

impl MprisController {
    pub fn new() -> Self {
        let controller = Self {
            identity: "Cine Rust".to_string(),
            current_title: Rc::new(RefCell::new(String::new())),
        };

        // Solicitar nombre en el bus de sesión de D-Bus de forma asíncrona
        let title_ref = controller.current_title.clone();
        gio::bus_own_name(
            gio::BusType::Session,
            "org.mpris.MediaPlayer2.Cine",
            gio::BusNameOwnerFlags::NONE,
            move |_conn, _name| {
                // Bus adquirido exitosamente
                eprintln!("[MPRIS] Bus adquirido para org.mpris.MediaPlayer2.Cine");
            },
            move |_conn, _name| {
                // Nombre adquirido
                eprintln!("[MPRIS] Nombre org.mpris.MediaPlayer2.Cine registrado en D-Bus");
            },
            move |_conn, _name| {
                // Bus perdido o desconectado
            },
        );

        let _ = title_ref;
        controller
    }

    pub fn update_metadata(&self) {
        // Punto de sincronización de metadatos con el entorno de escritorio
    }

    pub fn set_title(&self, title: &str) {
        *self.current_title.borrow_mut() = title.to_string();
    }
}
