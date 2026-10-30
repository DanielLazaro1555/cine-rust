// utils.rs
//
// Utilidades generales: rutas de configuración, formato de tiempo (MM:SS / HH:MM:SS),
// cálculo exacto del tamaño de ventana según resolución/aspecto del video,
// centrado automático de ventana en KDE/Wayland y búsqueda recursiva de archivos.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};
use std::process::Command;
use gtk::glib;

pub const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "webm", "avi", "mov", "flv", "wmv", "m4v", "mpg", "mpeg", "ts", "m2ts", "ogv",
    "3gp", "mp3", "flac", "wav", "ogg", "opus", "m4a",
];

/// Etiqueta legible para la calidad según la altura/anchura del video.
pub fn resolution_badge(width: i32, height: i32) -> String {
    let short_side = width.min(height);
    let long_side = width.max(height);
    if long_side >= 3800 && short_side >= 2000 {
        format!("4K ({width}×{height})")
    } else if long_side >= 2400 && short_side >= 1300 {
        format!("1440p ({width}×{height})")
    } else if long_side >= 1800 && short_side >= 950 {
        format!("1080p ({width}×{height})")
    } else if long_side >= 1200 && short_side >= 650 {
        format!("720p ({width}×{height})")
    } else if short_side >= 440 {
        format!("480p ({width}×{height})")
    } else if short_side >= 320 {
        format!("360p ({width}×{height})")
    } else if width > 0 && height > 0 {
        format!("{width}×{height}")
    } else {
        "Auto".to_string()
    }
}

/// Formatea segundos en `M:SS` o `H:MM:SS`, igual que `format_time` en `utils.py`.
pub fn format_time(seconds: f64) -> String {
    let total_secs = seconds.max(0.0).round() as u64;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes}:{secs:02}")
    }
}

/// Directorio de configuración de Cine (`~/.config/cine`).
pub fn config_dir() -> PathBuf {
    let mut dir = glib::user_config_dir();
    dir.push("cine");
    dir
}

/// Ruta del archivo de historial (`watch_history.jsonl`).
pub fn watch_history_path() -> PathBuf {
    let mut path = config_dir();
    path.push("watch_history.jsonl");
    path
}

/// Calcula las dimensiones óptimas de la ventana manteniendo exactamente la relación
/// de aspecto del video (algoritmo de precisión de `utils.py` de Cine para evitar bordes negros).
pub fn calculate_window_size(vid_w: i32, vid_h: i32, monitor_w: i32, monitor_h: i32) -> (i32, i32) {
    if vid_w <= 0 || vid_h <= 0 {
        return (1024, 576);
    }

    let mut width = vid_w as f64;
    let mut height = vid_h as f64;

    // Escalar videos muy pequeños como hace Cine original
    let scale_tiers = [(144.0, 1.6666), (240.0, 1.50), (360.0, 1.3333), (480.0, 1.15)];
    let min_dim = width.min(height);
    for (threshold, mult) in scale_tiers {
        if min_dim <= threshold {
            width *= mult;
            height *= mult;
            break;
        }
    }

    let monitor_area = (monitor_w as f64) * (monitor_h as f64);
    let occupy_factor = 0.60; // 60% del área de pantalla igual que DEFAULT_OCCUPY_SCREEN en Cine
    let video_area = width * height;
    let size_scale = ((monitor_area / video_area) * occupy_factor).sqrt();
    let target_scale = size_scale.min(1.0);

    let mut new_w = width * target_scale;
    let mut new_h = height * target_scale;

    let max_w = (monitor_w as f64 - 96.0).max(640.0);
    if new_w > max_w {
        new_w = max_w;
        new_h = height * new_w / width;
    }

    let max_h = (monitor_h as f64 - 106.0).max(480.0);
    if new_h > max_h {
        new_h = max_h;
        new_w = width * new_h / height;
    }

    let target_aspect = width / height;
    let mut final_w = new_w.round() as i32;
    let mut final_h = new_h.round() as i32;

    for _ in 0..20 {
        let current_aspect = (final_w as f64) / (final_h.max(1) as f64);
        if (current_aspect - target_aspect).abs() <= 0.00001 {
            break;
        }
        if current_aspect < target_aspect {
            final_w += 1;
        } else {
            final_h += 1;
        }
    }

    // Si el tamaño calculado es menor que el mínimo de la ventana, ampliar ambos
    // lados con la misma escala para conservar la relación de aspecto del video.
    let min_scale = (360.0 / final_w as f64)
        .max(202.0 / final_h as f64)
        .max(1.0);
    (
        (final_w as f64 * min_scale).round() as i32,
        (final_h as f64 * min_scale).round() as i32,
    )
}

/// Centra la ventana de Cine en KDE Plasma.
pub fn center_active_cine_window() {
    static CENTER_SCRIPT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    std::thread::spawn(|| {
        let _guard = CENTER_SCRIPT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        run_kwin_center_script("/tmp/cine_rust_center.js", "cine_rust_center", r#"
const cineIds = ["cine", "io.github.diegopvlk.cine"];
const candidates = workspace.stackingOrder.filter(w => {
    const rc = String(w.resourceClass || "").toLowerCase();
    const rn = String(w.resourceName || "").toLowerCase();
    return cineIds.includes(rc) || cineIds.includes(rn);
});
const c = candidates.find(w => w === workspace.activeWindow) || candidates[candidates.length - 1];
if (c && !c.dialog && !c.popupWindow) {
    const area = workspace.clientArea(KWin.PlacementArea, c);
    const fg = c.frameGeometry;
    c.frameGeometry = {x: Math.round(area.x + (area.width - fg.width) / 2), y: Math.round(area.y + (area.height - fg.height) / 2), width: fg.width, height: fg.height};
}
"#);
    });
}

/// Instala antes del primer mapeo un observador persistente de ventanas nuevas.
pub fn install_cine_window_added_centering() {
    static INSTALLED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    INSTALLED.get_or_init(|| {
        run_kwin_center_script("/tmp/cine_rust_center_on_add.js", "cine_rust_center_on_add", r#"
const cineIds = ["cine", "io.github.diegopvlk.cine"];
function centerCineWindow(c) {
    if (!c || c.deleted || c.dialog || c.popupWindow) return;
    const rc = String(c.resourceClass || "").toLowerCase();
    const rn = String(c.resourceName || "").toLowerCase();
    if (!cineIds.includes(rc) && !cineIds.includes(rn)) return;
    const area = workspace.clientArea(KWin.PlacementArea, c);
    const fg = c.frameGeometry;
    c.frameGeometry = {x: Math.round(area.x + (area.width - fg.width) / 2), y: Math.round(area.y + (area.height - fg.height) / 2), width: fg.width, height: fg.height};
}
workspace.windowAdded.connect(c => centerCineWindow(c));
for (const c of workspace.stackingOrder) centerCineWindow(c);
"#);
    });
}

fn run_kwin_center_script(script_path: &str, script_name: &str, script: &str) {
    if std::fs::write(script_path, script).is_err() { return; }
    let _ = Command::new("gdbus").args(["call", "--session", "--dest", "org.kde.KWin", "--object-path", "/Scripting", "--method", "org.kde.kwin.Scripting.unloadScript", script_name]).output();
    if let Ok(output) = Command::new("gdbus").args(["call", "--session", "--dest", "org.kde.KWin", "--object-path", "/Scripting", "--method", "org.kde.kwin.Scripting.loadScript", script_path, script_name]).output() {
        let id = String::from_utf8_lossy(&output.stdout).split(|c: char| !c.is_ascii_digit()).filter(|part| !part.is_empty()).last().unwrap_or("").to_string();
        if !id.is_empty() {
            let object_path = format!("/Scripting/Script{id}");
            let _ = Command::new("gdbus").args(["call", "--session", "--dest", "org.kde.KWin", "--object-path", &object_path, "--method", "org.kde.kwin.Script.run"]).output();
        }
    }
}

/// Verifica si un archivo tiene extensión multimedia soportada.
pub fn is_media_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| VIDEO_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

/// Recolecta todos los archivos multimedia dentro de un archivo o carpeta (ordenados por nombre).
pub fn collect_media_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let mut result = Vec::new();
    if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            let mut subdirs = Vec::new();
            let mut files = Vec::new();
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
                let p = entry.path();
                if p.is_file() && is_media_file(&p) {
                    files.push(p);
                } else if p.is_dir() {
                    subdirs.push(p);
                }
            }
            files.sort();
            subdirs.sort();
            result.extend(files);
            for sub in subdirs {
                result.extend(collect_media_files(&sub));
            }
        }
    }
    result
}
