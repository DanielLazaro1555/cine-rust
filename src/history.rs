// history.rs
//
// Historial interactivo de reproducción (`watch_history.jsonl`) y diálogo HistoryDialog.
// Registra las aperturas de archivos o URLs y permite reabrirlos con un solo clic.
// Equivalente modular a `history.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::rc::Rc;
use std::time::SystemTime;

use adw::prelude::*;

use crate::utils::watch_history_path;

#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub path: String,
    pub title: String,
    pub timestamp: u64,
}

/// Añade una entrada al historial persistente `watch_history.jsonl`.
pub fn record_history_entry(path: &str, title: &str) {
    let hist_path = watch_history_path();
    if let Some(parent) = hist_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&hist_path)
    else {
        return;
    };

    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Escape básico de comillas y barras
    let clean_path = path.replace('\\', "\\\\").replace('"', "\\\"");
    let clean_title = title.replace('\\', "\\\\").replace('"', "\\\"");

    let line = format!(
        "{{\"path\":\"{}\",\"title\":\"{}\",\"time\":{}}}\n",
        clean_path, clean_title, ts
    );
    let _ = file.write_all(line.as_bytes());
}

/// Lee las entradas del historial ordenadas de la más reciente a la más antigua.
pub fn read_history_entries() -> Vec<HistoryEntry> {
    let hist_path = watch_history_path();
    let Ok(file) = File::open(&hist_path) else {
        return Vec::new();
    };

    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines().flatten() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Extractor simple de campos JSON sin dependencias externas pesadas
        if let (Some(path), Some(title), time) = parse_json_line(line) {
            entries.push(HistoryEntry {
                path,
                title,
                timestamp: time,
            });
        }
    }

    // Invertir para que los más recientes aparezcan primero y limitar a 60
    entries.reverse();
    entries.truncate(60);
    entries
}

fn parse_json_line(line: &str) -> (Option<String>, Option<String>, u64) {
    let mut path = None;
    let mut title = None;
    let mut time = 0u64;

    if let Some(p_idx) = line.find("\"path\":\"") {
        let rest = &line[p_idx + 8..];
        if let Some(end) = rest.find("\",\"") {
            path = Some(rest[..end].replace("\\\"", "\"").replace("\\\\", "\\"));
        }
    }

    if let Some(t_idx) = line.find("\"title\":\"") {
        let rest = &line[t_idx + 9..];
        if let Some(end) = rest.find("\",\"") {
            title = Some(rest[..end].replace("\\\"", "\"").replace("\\\\", "\\"));
        }
    }

    if let Some(tm_idx) = line.find("\"time\":") {
        let rest = &line[tm_idx + 7..];
        let end = rest.find('}').unwrap_or(rest.len());
        if let Ok(val) = rest[..end].trim().parse::<u64>() {
            time = val;
        }
    }

    (path, title, time)
}

/// Borra el archivo de historial.
pub fn clear_history() {
    let hist_path = watch_history_path();
    let _ = std::fs::remove_file(hist_path);
}

/// Presenta el diálogo de historial de videos reproducidos.
pub fn present_history_dialog<FOpen>(parent: &impl IsA<gtk::Widget>, on_open: FOpen)
where
    FOpen: Fn(&str) + 'static,
{
    let dialog = adw::Dialog::builder()
        .title("Historial de reproducción")
        .content_width(540)
        .content_height(480)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();

    let entries = read_history_entries();

    if !entries.is_empty() {
        let clear_btn = gtk::Button::builder()
            .label("Borrar historial")
            .tooltip_text("Vaciar el registro de historial")
            .build();
        clear_btn.add_css_class("destructive-action");

        let dlg_weak = dialog.downgrade();
        clear_btn.connect_clicked(move |_| {
            clear_history();
            if let Some(dlg) = dlg_weak.upgrade() {
                dlg.close();
            }
        });
        header.pack_start(&clear_btn);
    }

    toolbar_view.add_top_bar(&header);

    if entries.is_empty() {
        let status = adw::StatusPage::builder()
            .icon_name("document-open-recent-symbolic")
            .title("Sin historial reciente")
            .description("Los videos reproducidos aparecerán aquí automáticamente")
            .build();
        toolbar_view.set_content(Some(&status));
    } else {
        let list_box = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        list_box.add_css_class("boxed-list");

        let f_open = Rc::new(on_open);
        for item in entries {
            let row = adw::ActionRow::builder()
                .title(&item.title)
                .subtitle(&item.path)
                .activatable(true)
                .build();

            let icon = gtk::Image::from_icon_name("cine-play-or-sub-symbolic");
            row.add_prefix(&icon);

            let f_op = f_open.clone();
            let p = item.path.clone();
            let dlg_weak = dialog.downgrade();
            row.connect_activated(move |_| {
                f_op(&p);
                if let Some(dlg) = dlg_weak.upgrade() {
                    dlg.close();
                }
            });

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
