// shortcuts.rs
//
// Atajos de teclado internos y diálogo de atajos (Shortcuts).
// Equivalente modular a `shortcuts.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::prelude::*;

/// Abre el diálogo de atajos de teclado desde el recurso compilado `/io/github/diegopvlk/Cine/shortcuts-dialog.ui`.
pub fn present_shortcuts_dialog(parent: &impl IsA<gtk::Widget>) {
    let builder = gtk::Builder::from_resource("/io/github/diegopvlk/Cine/shortcuts-dialog.ui");
    if let Some(dialog) = builder.object::<adw::Dialog>("shortcuts_dialog") {
        dialog.present(Some(parent));
    }
}

/// Registra los aceleradores de teclado principales en la aplicación.
pub fn register_window_accels(app: &adw::Application) {
    app.set_accels_for_action("win.open-folder", &["<Primary>i"]);
    app.set_accels_for_action("win.open-url", &["<Primary>u"]);
    app.set_accels_for_action("win.add-url", &["<Shift><Primary>u"]);
    app.set_accels_for_action("win.open-history", &["<Primary>h"]);
    app.set_accels_for_action("win.add-playlist-folder", &["<Shift><Primary>i"]);
    app.set_accels_for_action("win.open-playlist-dialog", &["<Primary>p"]);
    app.set_accels_for_action("win.clear-and-add", &["<Primary>o"]);
    app.set_accels_for_action("win.add-playlist-files", &["<Shift><Primary>o"]);
    app.set_accels_for_action("win.custom-shortcuts", &["<Primary>question"]);
    app.set_accels_for_action("win.quit", &["q", "<Primary>w"]);
}
