// options.rs
//
// Menú de opciones de video, subtítulos, rotación, zoom y velocidad.
// Equivalente modular a `options.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

/// Valores por defecto de transformación de video y velocidad de reproducción.
#[derive(Clone, Debug)]
pub struct PlaybackOptions {
    pub speed: f64,
    pub video_rotate: i32,
    pub video_zoom: f64,
    pub sub_delay: f64,
    pub audio_delay: f64,
}

impl Default for PlaybackOptions {
    fn default() -> Self {
        Self {
            speed: 1.0,
            video_rotate: 0,
            video_zoom: 0.0,
            sub_delay: 0.0,
            audio_delay: 0.0,
        }
    }
}
