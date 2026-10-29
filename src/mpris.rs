// mpris.rs
//
// Integración con el bus D-Bus `org.mpris.MediaPlayer2.Cine` para controles
// multimedia del escritorio (KDE / GNOME).
// Equivalente modular a `mpris.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Clone, Default)]
pub struct MprisController {
    pub identity: String,
}

impl MprisController {
    pub fn new() -> Self {
        Self {
            identity: "Cine".to_string(),
        }
    }

    pub fn update_metadata(&self) {
        // En los siguientes pasos se sincronizan metadatos D-Bus con el estado de libmpv
    }
}
