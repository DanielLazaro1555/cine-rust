// save_session.rs
//
// Guardado y restauración de la última lista de reproducción (save_session).
// Equivalente modular a `save_session.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use crate::utils::config_dir;

/// Ruta donde se almacena la última sesión (`last_playlist.m3u8`).
pub fn last_playlist_file() -> PathBuf {
    let mut path = config_dir();
    path.push("last_playlist.m3u8");
    path
}
