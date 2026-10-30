// playlist.rs
//
// Modelo y diálogo interactivo de lista de reproducción (Playlist).
// Soporta selección de pistas con clic, eliminación de elementos individuales y vaciado de lista.
// Equivalente modular a `playlist.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::rc::Rc;

use adw::prelude::*;

#[derive(Clone, Debug)]
pub struct PlaylistItem {
    pub path: PathBuf,
    pub title: String,
    pub playing: bool,
}

/// Presenta el diálogo interactivo de la lista de reproducción actual.
pub fn present_playlist_dialog<FSelect, FRemove, FClear, FAddFiles, FAddUrl>(
    parent: &impl IsA<gtk::Widget>,
    items: &[PlaylistItem],
    on_select: FSelect,
    on_remove: FRemove,
    on_clear: FClear,
    on_add_files: FAddFiles,
    on_add_url: FAddUrl,
) where
    FSelect: Fn(usize) + 'static,
    FRemove: Fn(usize) + 'static,
    FClear: Fn() + 'static,
    FAddFiles: Fn() + 'static,
    FAddUrl: Fn() + 'static,
{
    let dialog = adw::Dialog::builder()
        .title("Lista de reproducción")
        .content_width(540)
        .content_height(480)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();

    let f_add_files = Rc::new(on_add_files);
    let f_add_url = Rc::new(on_add_url);

    // Botón para añadir archivos locales a la lista
    let add_files_btn = gtk::Button::builder()
        .icon_name("cine-list-add-symbolic")
        .tooltip_text("Añadir archivos a la lista (Ctrl+Shift+O)")
        .build();
    add_files_btn.add_css_class("flat");
    {
        let f_af = f_add_files.clone();
        let dlg_weak = dialog.downgrade();
        add_files_btn.connect_clicked(move |_| {
            if let Some(dlg) = dlg_weak.upgrade() {
                dlg.close();
            }
            f_af();
        });
    }
    header.pack_start(&add_files_btn);

    // Botón para añadir enlace URL a la lista
    let add_url_btn = gtk::Button::builder()
        .icon_name("cine-globe-symbolic")
        .tooltip_text("Añadir enlace URL a la lista (Ctrl+Shift+U)")
        .build();
    add_url_btn.add_css_class("flat");
    {
        let f_au = f_add_url.clone();
        let dlg_weak = dialog.downgrade();
        add_url_btn.connect_clicked(move |_| {
            if let Some(dlg) = dlg_weak.upgrade() {
                dlg.close();
            }
            f_au();
        });
    }
    header.pack_start(&add_url_btn);

    // Botón para vaciar toda la lista
    if !items.is_empty() {
        let clear_btn = gtk::Button::builder()
            .label("Vaciar")
            .tooltip_text("Limpiar toda la lista de reproducción")
            .build();
        clear_btn.add_css_class("destructive-action");

        let dlg_weak = dialog.downgrade();
        let f_clear = Rc::new(on_clear);
        clear_btn.connect_clicked(move |_| {
            f_clear();
            if let Some(dlg) = dlg_weak.upgrade() {
                dlg.close();
            }
        });
        header.pack_end(&clear_btn);
    }

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
        let status = adw::StatusPage::builder()
            .icon_name("cine-playlist-symbolic")
            .title("La lista está vacía")
            .description("Añade videos locales o enlaces para reproducir en cola")
            .build();

        let empty_btn_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .halign(gtk::Align::Center)
            .margin_top(12)
            .build();

        let empty_files_btn = gtk::Button::builder()
            .label("Añadir archivos")
            .icon_name("cine-list-add-symbolic")
            .build();
        empty_files_btn.add_css_class("pill");
        {
            let f_af = f_add_files.clone();
            let dlg_weak = dialog.downgrade();
            empty_files_btn.connect_clicked(move |_| {
                if let Some(dlg) = dlg_weak.upgrade() {
                    dlg.close();
                }
                f_af();
            });
        }
        empty_btn_box.append(&empty_files_btn);

        let empty_url_btn = gtk::Button::builder()
            .label("Añadir URL")
            .icon_name("cine-globe-symbolic")
            .build();
        empty_url_btn.add_css_class("pill");
        {
            let f_au = f_add_url.clone();
            let dlg_weak = dialog.downgrade();
            empty_url_btn.connect_clicked(move |_| {
                if let Some(dlg) = dlg_weak.upgrade() {
                    dlg.close();
                }
                f_au();
            });
        }
        empty_btn_box.append(&empty_url_btn);

        status.set_child(Some(&empty_btn_box));
        toolbar_view.set_content(Some(&status));
    } else {
        let f_select = Rc::new(on_select);
        let f_remove = Rc::new(on_remove);

        for (idx, item) in items.iter().enumerate() {
            let row = adw::ActionRow::builder()
                .title(&item.title)
                .subtitle(item.path.to_string_lossy().as_ref())
                .activatable(true)
                .build();

            if item.playing {
                let icon = gtk::Image::from_icon_name("cine-playback-start-symbolic");
                icon.set_tooltip_text(Some("En reproducción"));
                row.add_prefix(&icon);
                row.add_css_class("accent");
            } else {
                let num_lbl = gtk::Label::builder()
                    .label(format!("{}.", idx + 1))
                    .width_request(24)
                    .build();
                num_lbl.add_css_class("dim-label");
                row.add_prefix(&num_lbl);
            }

            // Botón de eliminar elemento individual
            let remove_btn = gtk::Button::builder()
                .icon_name("edit-delete-symbolic")
                .tooltip_text("Eliminar de la lista")
                .valign(gtk::Align::Center)
                .build();
            remove_btn.add_css_class("flat");
            remove_btn.add_css_class("circular");

            {
                let f_rem = f_remove.clone();
                let dlg_weak = dialog.downgrade();
                remove_btn.connect_clicked(move |_| {
                    f_rem(idx);
                    if let Some(dlg) = dlg_weak.upgrade() {
                        dlg.close();
                    }
                });
            }
            row.add_suffix(&remove_btn);

            // Clic en la fila para reproducir inmediatamente
            {
                let f_sel = f_select.clone();
                let dlg_weak = dialog.downgrade();
                row.connect_activated(move |_| {
                    f_sel(idx);
                    if let Some(dlg) = dlg_weak.upgrade() {
                        dlg.close();
                    }
                });
            }

            list_box.append(&row);
        }

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&list_box)
            .build();
        toolbar_view.set_content(Some(&scrolled));
    }

    dialog.set_child(Some(&toolbar_view));
    dialog.present(Some(parent));
}
