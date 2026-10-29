// save_session.rs
//
// Guardado y restauración de la última lista de reproducción (save_session).
// Permite continuar la sesión anterior al abrir la aplicación si la opción está activa.
// Equivalente modular a `save_session.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::playlist::PlaylistItem;
use crate::utils::config_dir;

/// Ruta donde se almacena la última sesión (`last_playlist.m3u8`).
pub fn last_playlist_file() -> PathBuf {
    let mut path = config_dir();
    path.push("last_playlist.m3u8");
    path
}

/// Guarda los elementos de la lista actual en formato M3U8.
pub fn save_playlist_session(items: &[PlaylistItem]) {
    let path = last_playlist_file();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
    else {
        return;
    };

    let _ = writeln!(file, "#EXTM3U");
    for item in items {
        let _ = writeln!(file, "{}", item.path.to_string_lossy());
    }
}

/// Carga las rutas de la última sesión guardada.
pub fn load_playlist_session() -> Vec<PathBuf> {
    let path = last_playlist_file();
    let Ok(file) = File::open(&path) else {
        return Vec::new();
    };

    let reader = BufReader::new(file);
    let mut paths = Vec::new();

    for line in reader.lines().flatten() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        paths.push(PathBuf::from(line));
    }

    paths
}
