// history.rs
//
// Historial de reproducción (`watch_history.jsonl`) y diálogo HistoryDialog.
// Equivalente modular a `history.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::prelude::*;

use crate::utils::watch_history_path;

/// Presenta el diálogo de historial de videos reproducidos.
pub fn present_history_dialog(parent: &impl IsA<gtk::Widget>) {
    let dialog = adw::Dialog::builder()
        .title("Historial de reproducción")
        .content_width(500)
        .content_height(420)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&adw::HeaderBar::new());

    let path = watch_history_path();
    let status = adw::StatusPage::builder()
        .icon_name("document-open-recent-symbolic")
        .title("Historial")
        .description(format!("Archivo de historial: {}", path.display()))
        .build();

    toolbar_view.set_content(Some(&status));
    dialog.set_child(Some(&toolbar_view));
    dialog.present(Some(parent));
}
