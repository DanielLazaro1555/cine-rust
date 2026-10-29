// playlist.rs
//
// Modelo y diálogo de lista de reproducción (Playlist).
// Equivalente modular a `playlist.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use adw::prelude::*;

#[derive(Clone, Debug)]
pub struct PlaylistItem {
    pub path: PathBuf,
    pub title: String,
    pub playing: bool,
}

/// Presenta el diálogo de lista de reproducción actual.
pub fn present_playlist_dialog(parent: &impl IsA<gtk::Widget>, items: &[PlaylistItem]) {
    let dialog = adw::Dialog::builder()
        .title("Lista de reproducción")
        .content_width(480)
        .content_height(420)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();
    toolbar_view.add_top_bar(&header);

    let list_box = gtk::ListBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    list_box.add_css_class("boxed-list");

    if items.is_empty() {
        let row = adw::ActionRow::builder()
            .title("La lista de reproducción está vacía")
            .build();
        list_box.append(&row);
    } else {
        for item in items {
            let row = adw::ActionRow::builder()
                .title(&item.title)
                .subtitle(item.path.to_string_lossy().as_ref())
                .build();
            if item.playing {
                let icon = gtk::Image::from_icon_name("cine-playback-start-symbolic");
                row.add_prefix(&icon);
            }
            list_box.append(&row);
        }
    }

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&list_box)
        .build();
    toolbar_view.set_content(Some(&scrolled));
    dialog.set_child(Some(&toolbar_view));

    dialog.present(Some(parent));
}
