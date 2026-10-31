// utils.rs
//
// Utilidades generales: rutas de configuración, formato de tiempo (MM:SS / HH:MM:SS),
// cálculo exacto del tamaño de ventana según resolución/aspecto del video,
// centrado automático de ventana en KDE/Wayland y búsqueda recursiva de archivos.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::c_void;
use std::os::raw::{c_int, c_ulong};
use std::path::{Path, PathBuf};

static CENTER_SCRIPT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
use std::process::Command;
use gtk::glib;

#[link(name = "gtk-4")]
unsafe extern "C" {
    fn gdk_x11_display_get_type() -> usize;
    fn gdk_x11_display_get_xdisplay(display: *mut c_void) -> *mut c_void;
    fn gdk_x11_surface_get_type() -> usize;
    fn gdk_x11_surface_get_xid(surface: *mut c_void) -> c_ulong;
}

#[link(name = "gobject-2.0")]
unsafe extern "C" {
    fn g_type_check_instance_is_a(instance: *mut c_void, g_type: usize) -> c_int;
}

#[link(name = "libX11.so.6", modifiers = "+verbatim")]
unsafe extern "C" {
    fn XMoveWindow(display: *mut c_void, window: c_ulong, x: c_int, y: c_int) -> c_int;
    fn XTranslateCoordinates(
        display: *mut c_void,
        src_w: c_ulong,
        dest_w: c_ulong,
        src_x: c_int,
        src_y: c_int,
        dest_x: *mut c_int,
        dest_y: *mut c_int,
        child: *mut c_ulong,
    ) -> c_int;
    fn XDefaultRootWindow(display: *mut c_void) -> c_ulong;
    fn XFlush(display: *mut c_void) -> c_int;
}

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

/// Calcula un tamaño proporcional al video y al área del monitor disponible.
pub fn calculate_window_size(
    vid_w: i32,
    vid_h: i32,
    monitor_w: i32,
    monitor_h: i32,
    chrome_w: i32,
    chrome_h: i32,
) -> (i32, i32) {
    if vid_w <= 0 || vid_h <= 0 {
        return (1024, 576);
    }

    let video_w = vid_w as f64;
    let video_h = vid_h as f64;
    let screen_w = monitor_w.max(1) as f64;
    let screen_h = monitor_h.max(1) as f64;

    // Mantener márgenes proporcionales en pantallas pequeñas y grandes.
    let max_w = (screen_w * 0.90 - chrome_w.max(0) as f64).max(360.0);
    let max_h = (screen_h * 0.82 - chrome_h.max(0) as f64).max(202.0);
    let area_scale = ((screen_w * screen_h * 0.60) / (video_w * video_h)).sqrt();
    let fit_scale = (max_w / video_w).min(max_h / video_h);
    let min_scale = (360.0 / video_w).max(202.0 / video_h);
    let scale = area_scale.min(fit_scale).max(min_scale);

    (
        (video_w * scale).round() as i32,
        (video_h * scale).round() as i32,
    )
}

/// Centra la ventana usando el área de trabajo que KWin asigna a esa ventana.
/// Así evitamos mezclar coordenadas lógicas de GTK con píxeles del monitor.
pub fn center_cine_window() {
    let app_pid = std::process::id();
    std::thread::spawn(move || {
        let _guard = CENTER_SCRIPT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        let script = r#"
const appPid = __APP_PID__;
const candidates = workspace.stackingOrder.filter(w =>
    w && !w.deleted && !w.dialog && !w.popupWindow && Number(w.pid) === appPid
);
const active = workspace.activeWindow;
const c = candidates.find(w => active && String(w.internalId) === String(active.internalId)) || candidates[0];
if (!c) {
    print("[CineRust] KWin no encontró una ventana para PID " + appPid);
} else {
    workspace.activeWindow = c;
    workspace.raiseWindow(c);
    const area = workspace.clientArea(KWin.WorkArea, c);
    const fg = c.frameGeometry;
    const targetX = Math.round(area.x + (area.width - fg.width) / 2);
    const targetY = Math.round(area.y + (area.height - fg.height) / 2);
    // En KWin 6 hay que cambiar x/y por separado; asignar el rectángulo
    // completo no desplaza la ventana.
    c.frameGeometry.x = targetX;
    c.frameGeometry.y = targetY;
    print("[CineRust] KWin centró PID " + appPid + ": actual=" + JSON.stringify(c.frameGeometry) + ", objetivo=" + targetX + "," + targetY + ", área=" + JSON.stringify(area));
}

"#
        .replace("__APP_PID__", &app_pid.to_string());
        run_kwin_center_script("/tmp/cine_rust_center.js", "cine_rust_center", &script);
    });
}

/// Centra una superficie X11/XWayland con posición absoluta.
pub fn center_x11_surface(display: &gtk::gdk::Display, surface: &gtk::gdk::Surface) -> bool {
    use glib::object::ObjectType;
    use gtk::prelude::{DisplayExt, MonitorExt, SurfaceExt};

    let Some(monitor) = display.monitor_at_surface(surface) else {
        return false;
    };
    let display_ptr = display.as_ptr() as *mut c_void;
    let surface_ptr = surface.as_ptr() as *mut c_void;
    unsafe {
        if g_type_check_instance_is_a(display_ptr, gdk_x11_display_get_type()) == 0
            || g_type_check_instance_is_a(surface_ptr, gdk_x11_surface_get_type()) == 0
        {
            return false;
        }

        let xdisplay = gdk_x11_display_get_xdisplay(display_ptr);
        let xid = gdk_x11_surface_get_xid(surface_ptr);
        if xdisplay.is_null() || xid == 0 {
            return false;
        }

        let area = monitor.geometry();
        let x = area.x() + (area.width() - surface.width()) / 2;
        let y = area.y() + (area.height() - surface.height()) / 2;
        XMoveWindow(xdisplay, xid, x, y);
        XFlush(xdisplay);
        eprintln!(
            "[CineRust] Centrado X11 aplicado: ventana {xid:#x}, posición ({x},{y}), monitor {}x{}",
            area.width(),
            area.height()
        );
        true
    }
}

/// Obtiene/establece la posición absoluta de una superficie X11/XWayland.
pub fn x11_surface_position(
    display: &gtk::gdk::Display,
    surface: &gtk::gdk::Surface,
) -> Option<(i32, i32)> {
    use glib::object::ObjectType;
    let display_ptr = display.as_ptr() as *mut c_void;
    let surface_ptr = surface.as_ptr() as *mut c_void;
    unsafe {
        if g_type_check_instance_is_a(display_ptr, gdk_x11_display_get_type()) == 0
            || g_type_check_instance_is_a(surface_ptr, gdk_x11_surface_get_type()) == 0
        {
            return None;
        }
        let xdisplay = gdk_x11_display_get_xdisplay(display_ptr);
        let xid = gdk_x11_surface_get_xid(surface_ptr);
        if xdisplay.is_null() || xid == 0 { return None; }
        let mut x = 0;
        let mut y = 0;
        let mut child = 0;
        (XTranslateCoordinates(xdisplay, xid, XDefaultRootWindow(xdisplay), 0, 0, &mut x, &mut y, &mut child) != 0).then_some((x, y))
    }
}

pub fn move_x11_surface(
    display: &gtk::gdk::Display,
    surface: &gtk::gdk::Surface,
    x: i32,
    y: i32,
) -> bool {
    use glib::object::ObjectType;
    let display_ptr = display.as_ptr() as *mut c_void;
    let surface_ptr = surface.as_ptr() as *mut c_void;
    unsafe {
        if g_type_check_instance_is_a(display_ptr, gdk_x11_display_get_type()) == 0
            || g_type_check_instance_is_a(surface_ptr, gdk_x11_surface_get_type()) == 0 { return false; }
        let xdisplay = gdk_x11_display_get_xdisplay(display_ptr);
        let xid = gdk_x11_surface_get_xid(surface_ptr);
        if xdisplay.is_null() || xid == 0 { return false; }
        XMoveWindow(xdisplay, xid, x, y);
        XFlush(xdisplay);
        true
    }
}

pub fn save_window_position(x: i32, y: i32) {
    let dir = config_dir();
    if std::fs::create_dir_all(&dir).is_err() { return; }
    let path = dir.join("window.conf");
    let keyfile = glib::KeyFile::new();
    keyfile.set_integer("Window", "x", x);
    keyfile.set_integer("Window", "y", y);
    let _ = keyfile.save_to_file(path);
}

pub fn load_window_position() -> Option<(i32, i32)> {
    let keyfile = glib::KeyFile::new();
    keyfile.load_from_file(config_dir().join("window.conf"), glib::KeyFileFlags::NONE).ok()?;
    Some((keyfile.integer("Window", "x").ok()?, keyfile.integer("Window", "y").ok()?))
}

fn run_kwin_center_script(script_path: &str, script_name: &str, script: &str) {
    if let Err(error) = std::fs::write(script_path, script) {
        eprintln!("[CineRust] No se pudo escribir el script de centrado: {error}");
        return;
    }
    let unload = Command::new("gdbus")
        .args(["call", "--session", "--dest", "org.kde.KWin", "--object-path", "/Scripting", "--method", "org.kde.kwin.Scripting.unloadScript", script_name])
        .output();
    if let Err(error) = unload {
        eprintln!("[CineRust] No se pudo contactar KWin para descargar el script: {error}");
    }
    let output = match Command::new("gdbus")
        .args(["call", "--session", "--dest", "org.kde.KWin", "--object-path", "/Scripting", "--method", "org.kde.kwin.Scripting.loadScript", script_path, script_name])
        .output()
    {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            eprintln!("[CineRust] KWin rechazó el script de centrado: {}", String::from_utf8_lossy(&output.stderr));
            return;
        }
        Err(error) => {
            eprintln!("[CineRust] No se pudo iniciar gdbus para centrar la ventana: {error}");
            return;
        }
    };
    let id = String::from_utf8_lossy(&output.stdout)
        .split(|c: char| !c.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .last()
        .unwrap_or("")
        .to_string();
    if id.is_empty() {
        eprintln!("[CineRust] KWin no devolvió el ID del script de centrado.");
        return;
    }
    let object_path = format!("/Scripting/Script{id}");
    match Command::new("gdbus")
        .args(["call", "--session", "--dest", "org.kde.KWin", "--object-path", &object_path, "--method", "org.kde.kwin.Script.run"])
        .output()
    {
        Ok(output) if output.status.success() => {}
        Ok(output) => eprintln!("[CineRust] KWin no pudo ejecutar el script: {}", String::from_utf8_lossy(&output.stderr)),
        Err(error) => eprintln!("[CineRust] No se pudo ejecutar el script de centrado: {error}"),
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
        return if is_media_file(path) { vec![path.to_path_buf()] } else { Vec::new() };
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
