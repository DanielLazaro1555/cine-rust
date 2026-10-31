// mpv_gl_area.rs
//
// Área de renderizado OpenGL (`VideoGLArea`) que conecta `GtkGLArea` con `MpvEngine`.
// Equivalente modular a `mpv_gl_area.py`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};
use std::ffi::c_void;
use std::rc::Rc;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Duration, Instant};

use gtk::glib;
use gtk::prelude::*;

use crate::mpv_engine::MpvEngine;

/// Callback invocado desde el hilo de renderizado de `libmpv` cuando hay un nuevo
/// fotograma listo para dibujarse en el `GtkGLArea`.
struct MpvRenderCallbackContext {
    render_queued: Arc<AtomicBool>,
    stats: Arc<Mutex<RenderStats>>,
}

#[derive(Default)]
struct RenderStats {
    update_callbacks: u64,
    queued_redraws: u64,
    renders: u64,
    render_time: Duration,
    max_render_time: Duration,
    queue_delay: Duration,
    max_queue_delay: Duration,
    pending_since: Option<Instant>,
    last_report: Option<Instant>,
}

unsafe extern "C" fn on_mpv_render_update(ctx: *mut c_void) {
    if ctx.is_null() {
        return;
    }
    let callback_context = unsafe { &*(ctx as *const MpvRenderCallbackContext) };
    if let Ok(mut stats) = callback_context.stats.lock() {
        stats.update_callbacks += 1;
    }
    // mpv puede notificar varias veces antes de que GTK alcance a dibujar.
    // Agrupa esos avisos para no acumular renders viejos en la cola de eventos.
    if callback_context.render_queued.swap(true, Ordering::AcqRel) {
        return;
    }
    if let Ok(mut stats) = callback_context.stats.lock() {
        stats.queued_redraws += 1;
        stats.pending_since = Some(Instant::now());
    }
}

#[derive(Clone)]
pub struct VideoGLArea {
    pub gl_area: gtk::GLArea,
    pub offload: gtk::GraphicsOffload,
    pub mpv: Rc<RefCell<Option<MpvEngine>>>,
}

impl VideoGLArea {
    pub fn new(enable_offload: bool, hwdec: bool, volume: i32) -> Self {
        let gl_area = gtk::GLArea::builder()
            .hexpand(true)
            .vexpand(true)
            .auto_render(false)
            .build();

        let offload = gtk::GraphicsOffload::new(Some(&gl_area));
        offload.set_black_background(true);
        offload.set_enabled(if enable_offload {
            gtk::GraphicsOffloadEnabled::Enabled
        } else {
            gtk::GraphicsOffloadEnabled::Disabled
        });

        let mpv = Rc::new(RefCell::new(MpvEngine::new(hwdec, volume)));
        let fbo_width = Rc::new(Cell::new(1920));
        let fbo_height = Rc::new(Cell::new(1080));
        let cb_ctx_ptr = Rc::new(Cell::new(std::ptr::null_mut()));
        let render_queued = Arc::new(AtomicBool::new(false));
        let render_stats = Arc::new(Mutex::new(RenderStats::default()));

        // Actualizar dimensiones FBO en resize y cambio de escala HiDPI
        {
            let fbo_w = fbo_width.clone();
            let fbo_h = fbo_height.clone();
            gl_area.connect_resize(move |area, width, height| {
                let scale = area.scale_factor();
                fbo_w.set((width * scale).max(1));
                fbo_h.set((height * scale).max(1));
            });
        }

        // Inicializar el contexto OpenGL de libmpv al realizar el widget
        {
            let mpv = mpv.clone();
            let cb_ctx_ptr = cb_ctx_ptr.clone();
            let callback_render_queued = Arc::clone(&render_queued);
            let callback_render_stats = Arc::clone(&render_stats);
            gl_area.connect_realize(move |area| {
                area.make_current();
                if area.error().is_some() {
                    return;
                }
                let callback_context = Box::new(MpvRenderCallbackContext {
                    render_queued: callback_render_queued.clone(),
                    stats: callback_render_stats.clone(),
                });
                let raw_ptr = Box::into_raw(callback_context) as *mut c_void;
                cb_ctx_ptr.set(raw_ptr);

                if let Some(engine) = mpv.borrow_mut().as_mut() {
                    engine.init_gl_context(on_mpv_render_update, raw_ptr);
                }
            });
        }

        // Renderizar fotograma de libmpv en el FBO actual de GTK4
        {
            let mpv = mpv.clone();
            let fbo_w = fbo_width.clone();
            let fbo_h = fbo_height.clone();
            let render_stats = Arc::clone(&render_stats);
            gl_area.connect_render(move |_area, _context| {
                let render_started = Instant::now();
                if let Some(engine) = mpv.borrow().as_ref() {
                    engine.render_frame(fbo_w.get(), fbo_h.get());
                }
                let render_time = render_started.elapsed();
                let report = if let Ok(mut stats) = render_stats.lock() {
                    stats.renders += 1;
                    stats.render_time += render_time;
                    stats.max_render_time = stats.max_render_time.max(render_time);
                    if let Some(pending_since) = stats.pending_since.take() {
                        let delay = render_started.saturating_duration_since(pending_since);
                        stats.queue_delay += delay;
                        stats.max_queue_delay = stats.max_queue_delay.max(delay);
                    }

                    let now = Instant::now();
                    let should_report = stats.last_report
                        .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(5));
                    if should_report {
                        let snapshot = (
                            stats.update_callbacks,
                            stats.queued_redraws,
                            stats.renders,
                            stats.render_time,
                            stats.max_render_time,
                            stats.queue_delay,
                            stats.max_queue_delay,
                        );
                        *stats = RenderStats { last_report: Some(now), ..RenderStats::default() };
                        Some(snapshot)
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some((updates, queued, renders, total_render_time, max_render_time, total_queue_delay, max_queue_delay)) = report {
                    // Never call libmpv while holding render_stats: its update
                    // callback may need that mutex while the core serves reads.
                    let mpv_stats = if let Some(engine) = mpv.borrow().as_ref() {
                        [
                            "decoder-frame-drop-count",
                            "frame-drop-count",
                            "vo-delayed-frame-count",
                            "mistimed-frame-count",
                            "display-fps",
                            "estimated-display-fps",
                            "vsync-ratio",
                            "avsync",
                            "display-sync-active",
                        ]
                            .map(|property| engine.get_property_str(property).unwrap_or_else(|| "?".into()))
                    } else {
                        std::array::from_fn(|_| "?".to_owned())
                    };
                    let render_count = renders.max(1);
                    eprintln!(
                        "[CineRust] Render 5s: avisos={updates}, solicitudes GTK={queued}, renders={renders}, promedio={:.2}ms, máximo={:.2}ms, espera promedio={:.2}ms, espera máxima={:.2}ms; mpv acumulado: descartados decodificador/VO={}/{}, retrasados={}, fuera de tiempo={}; pantalla: fps={}, estimado={}, ratio-vsync={}, A/V={}s, display-sync={}",
                        total_render_time.as_secs_f64() * 1000.0 / render_count as f64,
                        max_render_time.as_secs_f64() * 1000.0,
                        total_queue_delay.as_secs_f64() * 1000.0 / render_count as f64,
                        max_queue_delay.as_secs_f64() * 1000.0,
                        mpv_stats[0], mpv_stats[1], mpv_stats[2], mpv_stats[3],
                        mpv_stats[4], mpv_stats[5], mpv_stats[6], mpv_stats[7], mpv_stats[8],
                    );
                }
                glib::Propagation::Stop
            });
        }

        // Consume mpv redraw requests on GTK's frame clock. Scheduling a low
        // priority GLib idle for every video update can add uneven latency and
        // makes the GLArea miss display frames under event-loop load.
        {
            let render_queued = Arc::clone(&render_queued);
            gl_area.add_tick_callback(move |area, _frame_clock| {
                if render_queued.swap(false, Ordering::AcqRel) {
                    area.queue_render();
                }
                glib::ControlFlow::Continue
            });
        }

        // Liberar limpiamente el contexto OpenGL y libmpv al destruir el área
        {
            let mpv = mpv.clone();
            let cb_ctx_ptr = cb_ctx_ptr.clone();
            gl_area.connect_unrealize(move |area| {
                area.make_current();
                if let Some(mut engine) = mpv.borrow_mut().take() {
                    engine.shutdown();
                }
                let ptr = cb_ctx_ptr.replace(std::ptr::null_mut());
                if !ptr.is_null() {
                    unsafe {
                        let _ = Box::from_raw(ptr as *mut MpvRenderCallbackContext);
                    }
                }
            });
        }

        Self {
            gl_area,
            offload,
            mpv,
        }
    }

    pub fn widget(&self) -> &gtk::GraphicsOffload {
        &self.offload
    }

    pub fn set_visible(&self, visible: bool) {
        self.gl_area.set_visible(visible);
    }

    pub fn load_file(&self, path: &str, mode: &str) {
        self.set_video_pan(0.0, 0.0);
        if let Some(engine) = self.mpv.borrow().as_ref() {
            engine.command(&["loadfile", path, mode]);
        }
    }

    pub fn toggle_pause(&self) -> bool {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            engine.command_sync(&["cycle", "pause"]);
            return engine.get_property_bool("pause").unwrap_or(false);
        }
        false
    }

    pub fn seek_absolute(&self, seconds: f64) {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            let s = format!("{seconds:.2}");
            engine.command(&["seek", &s, "absolute"]);
        }
    }

    pub fn seek_relative(&self, delta_seconds: f64) {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            let s = format!("{delta_seconds:.2}");
            engine.command(&["seek", &s, "relative"]);
        }
    }

    pub fn set_volume(&self, volume: f64) {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            let v = format!("{volume:.0}");
            engine.command(&["set", "volume", &v]);
        }
    }

    pub fn command(&self, args: &[&str]) {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            engine.command(args);
        }
    }

    pub fn set_property(&self, name: &str, val: &str) {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            engine.command(&["set", name, val]);
        }
    }

    pub fn set_speed(&self, speed: f64) {
        self.set_property("speed", &format!("{speed:.2}"));
    }

    pub fn set_aspect_ratio(&self, aspect: &str) {
        self.set_property("video-aspect-override", aspect);
    }

    pub fn set_video_rotate(&self, deg: i32) {
        self.set_property("video-rotate", &deg.to_string());
        // The pan bounds depend on the displayed dimensions. A quarter turn
        // changes those dimensions, so retaining the old pan can leave the
        // newly rotated frame already pressed against one of its new bounds.
        self.set_video_pan(0.0, 0.0);
    }

    pub fn set_video_zoom(&self, zoom: f64) {
        self.set_property("video-zoom", &format!("{zoom:.2}"));
        if zoom <= 0.0 {
            self.set_video_pan(0.0, 0.0);
        }
    }

    pub fn reset_video_view(&self) {
        self.set_video_zoom(0.0);
        self.set_video_pan(0.0, 0.0);
    }

    pub fn video_pan(&self) -> (f64, f64) {
        let x = self.get_property_str("video-pan-x").and_then(|v| v.parse().ok()).unwrap_or(0.0);
        let y = self.get_property_str("video-pan-y").and_then(|v| v.parse().ok()).unwrap_or(0.0);
        (x, y)
    }

    pub fn set_video_pan(&self, x: f64, y: f64) {
        self.set_property("video-pan-x", &format!("{x:.4}"));
        self.set_property("video-pan-y", &format!("{y:.4}"));
    }

    pub fn video_pan_for_drag(
        &self,
        start_x: f64,
        start_y: f64,
        dx: f64,
        dy: f64,
        zoom: f64,
    ) -> (f64, f64) {
        let Some((video_w, video_h)) = self.video_dimensions() else {
            return (start_x, start_y);
        };
        let area_w = self.gl_area.width() as f64;
        let area_h = self.gl_area.height() as f64;
        if area_w <= 0.0 || area_h <= 0.0 {
            return (start_x, start_y);
        }

        let fit_scale = (area_w / video_w as f64).min(area_h / video_h as f64);
        let zoom_scale = 2.0_f64.powf(zoom);
        let displayed_w = video_w as f64 * fit_scale * zoom_scale;
        let displayed_h = video_h as f64 * fit_scale * zoom_scale;
        let pan_x = start_x + dx / displayed_w;
        let pan_y = start_y + dy / displayed_h;
        self.clamp_video_pan(pan_x, pan_y, zoom)
    }

    pub fn clamp_video_pan(&self, x: f64, y: f64, zoom: f64) -> (f64, f64) {
        let Some((video_w, video_h)) = self.video_dimensions() else {
            return (0.0, 0.0);
        };
        let area_w = self.gl_area.width() as f64;
        let area_h = self.gl_area.height() as f64;
        if area_w <= 0.0 || area_h <= 0.0 {
            return (0.0, 0.0);
        }

        let fit_scale = (area_w / video_w as f64).min(area_h / video_h as f64);
        let display_w = video_w as f64 * fit_scale * 2.0_f64.powf(zoom);
        let display_h = video_h as f64 * fit_scale * 2.0_f64.powf(zoom);
        let max_pan_x = ((display_w - area_w) / (2.0 * display_w)).max(0.0);
        let max_pan_y = ((display_h - area_h) / (2.0 * display_h)).max(0.0);

        (x.clamp(-max_pan_x, max_pan_x), y.clamp(-max_pan_y, max_pan_y))
    }

    pub fn set_normalize_volume(&self, enabled: bool) {
        if enabled {
            self.command(&[
                "af",
                "add",
                "@cine_loudnorm:lavfi=[loudnorm=I=-20]",
            ]);
        } else {
            self.command(&["af", "remove", "@cine_loudnorm"]);
        }
    }

    pub fn set_sub_delay(&self, delay: f64) {
        self.set_property("sub-delay", &format!("{delay:.2}"));
    }

    pub fn set_audio_delay(&self, delay: f64) {
        self.set_property("audio-delay", &format!("{delay:.2}"));
    }

    pub fn set_contrast(&self, val: i32) {
        self.set_property("contrast", &val.to_string());
    }

    pub fn set_brightness(&self, val: i32) {
        self.set_property("brightness", &val.to_string());
    }

    pub fn set_gamma(&self, val: i32) {
        self.set_property("gamma", &val.to_string());
    }

    pub fn set_saturation(&self, val: i32) {
        self.set_property("saturation", &val.to_string());
    }

    pub fn set_hue(&self, val: i32) {
        self.set_property("hue", &val.to_string());
    }

    pub fn set_loop_file(&self, loop_mode: bool) {
        self.set_property("loop-file", if loop_mode { "inf" } else { "no" });
    }

    pub fn set_loop_playlist(&self, loop_mode: bool) {
        self.set_property("loop-playlist", if loop_mode { "inf" } else { "no" });
    }

    pub fn get_property_str(&self, name: &str) -> Option<String> {
        self.mpv.borrow().as_ref().and_then(|e| e.get_property_str(name))
    }

    pub fn get_property_bool(&self, name: &str) -> Option<bool> {
        self.mpv.borrow().as_ref().and_then(|e| e.get_property_bool(name))
    }

    pub fn toggle_stats(&self) {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            engine.command(&["script-binding", "stats/display-stats-toggle"]);
        }
    }

    pub fn time_pos_and_duration(&self) -> (f64, f64, bool) {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            let pos = engine.get_property_f64("time-pos").unwrap_or(0.0);
            let dur = engine.get_property_f64("duration").unwrap_or(0.0);
            let paused = engine.get_property_bool("pause").unwrap_or(false);
            return (pos, dur, paused);
        }
        (0.0, 0.0, true)
    }

    /// Obtiene las dimensiones reales de visualización (`dwidth` x `dheight`) del video actual
    /// (incluyendo relación de aspecto de píxeles y la rotación manual de mpv).
    pub fn video_dimensions(&self) -> Option<(i32, i32)> {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            let mut dw = engine.get_property_f64("dwidth")? as i32;
            let mut dh = engine.get_property_f64("dheight")? as i32;
            let rotation = engine
                .get_property_f64("video-rotate")
                .unwrap_or(0.0)
                .round() as i32;

            // dwidth/dheight include display aspect and video filters, but
            // mpv's manual video-rotate is a separate display transform.
            // Swap the axes for quarter turns so pan bounds match the frame.
            if rotation.rem_euclid(180) == 90 {
                std::mem::swap(&mut dw, &mut dh);
            }
            if dw > 0 && dh > 0 {
                return Some((dw, dh));
            }
        }
        None
    }

    /// Devuelve información detallada de la calidad del video en reproducción:
    /// `(width, height, fps, codec, hwdec)` y el título real del medio (`media-title`).
    pub fn video_quality_info(&self) -> (Option<(i32, i32, f64, String, String)>, Option<String>) {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            let media_title = engine.get_property_str("media-title");
            if let (Some(w), Some(h)) = (
                engine.get_property_f64("width"),
                engine.get_property_f64("height"),
            ) {
                let wi = w as i32;
                let hi = h as i32;
                if wi > 0 && hi > 0 {
                    let fps = engine.get_property_f64("container-fps").unwrap_or(0.0);
                    let codec = engine
                        .get_property_str("video-format")
                        .unwrap_or_else(|| "video".to_string())
                        .to_uppercase();
                    let hwdec = engine
                        .get_property_str("hwdec-current")
                        .unwrap_or_else(|| "sw".to_string());
                    return (Some((wi, hi, fps, codec, hwdec)), media_title);
                }
            }
            return (None, media_title);
        }
        (None, None)
    }

    /// Cambia el formato de calidad de `yt-dlp` (por ejemplo 4K, 1080p, 720p, 480p).
    pub fn set_ytdl_format(&self, format_str: &str) {
        if let Some(engine) = self.mpv.borrow().as_ref() {
            engine.set_property_str("ytdl-format", format_str);
        }
    }

    pub fn shutdown(&self) {
        if let Some(mut engine) = self.mpv.borrow_mut().take() {
            engine.shutdown();
        }
    }
}
