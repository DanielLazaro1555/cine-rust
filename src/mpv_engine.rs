// mpv_engine.rs
//
// Enlace directo FFI en Rust con `libmpv`, `libEGL` y `libGL`.
// Permite reproducir video y audio con aceleración por hardware (`hwdec=auto`)
// renderizando directamente en el Framebuffer OpenGL de `GtkGLArea`.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_void};
use std::ptr;

#[repr(C)]
pub struct MpvHandle {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MpvRenderContext {
    _private: [u8; 0],
}

#[repr(C)]
pub struct MpvRenderParam {
    pub type_: c_int,
    pub data: *mut c_void,
}

#[repr(C)]
pub struct MpvOpenGlInitParams {
    pub get_proc_address: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> *mut c_void>,
    pub get_proc_address_ctx: *mut c_void,
}

#[repr(C)]
pub struct MpvOpenGlFbo {
    pub fbo: c_int,
    pub w: c_int,
    pub h: c_int,
    pub internal_format: c_int,
}

pub const MPV_RENDER_PARAM_INVALID: c_int = 0;
pub const MPV_RENDER_PARAM_API_TYPE: c_int = 1;
pub const MPV_RENDER_PARAM_OPENGL_INIT_PARAMS: c_int = 2;
pub const MPV_RENDER_PARAM_OPENGL_FBO: c_int = 3;
pub const MPV_RENDER_PARAM_FLIP_Y: c_int = 4;

pub const GL_FRAMEBUFFER_BINDING: c_uint = 0x8CA6;

pub type MpvRenderUpdateFn = unsafe extern "C" fn(cb_ctx: *mut c_void);

#[link(name = "mpv")]
unsafe extern "C" {
    pub fn mpv_create() -> *mut MpvHandle;
    pub fn mpv_initialize(ctx: *mut MpvHandle) -> c_int;
    pub fn mpv_set_option_string(
        ctx: *mut MpvHandle,
        name: *const c_char,
        data: *const c_char,
    ) -> c_int;
    pub fn mpv_command(ctx: *mut MpvHandle, args: *const *const c_char) -> c_int;
    pub fn mpv_command_async(
        ctx: *mut MpvHandle,
        reply_userdata: u64,
        args: *const *const c_char,
    ) -> c_int;
    pub fn mpv_get_property_string(ctx: *mut MpvHandle, name: *const c_char) -> *mut c_char;
    pub fn mpv_free(data: *mut c_void);
    pub fn mpv_terminate_destroy(ctx: *mut MpvHandle);

    pub fn mpv_render_context_create(
        res: *mut *mut MpvRenderContext,
        mpv: *mut MpvHandle,
        params: *mut MpvRenderParam,
    ) -> c_int;
    pub fn mpv_render_context_set_update_callback(
        ctx: *mut MpvRenderContext,
        callback: Option<MpvRenderUpdateFn>,
        callback_ctx: *mut c_void,
    );
    pub fn mpv_render_context_render(
        ctx: *mut MpvRenderContext,
        params: *mut MpvRenderParam,
    ) -> c_int;
    pub fn mpv_render_context_free(ctx: *mut MpvRenderContext);
}

#[link(name = "EGL")]
unsafe extern "C" {
    pub fn eglGetProcAddress(procname: *const c_char) -> *mut c_void;
}

#[link(name = "GL")]
unsafe extern "C" {
    pub fn glGetIntegerv(pname: c_uint, data: *mut c_int);
}

unsafe extern "C" fn get_proc_address_trampoline(
    _ctx: *mut c_void,
    name: *const c_char,
) -> *mut c_void {
    unsafe { eglGetProcAddress(name) }
}

/// Motor seguro que encapsula `mpv_handle` y `mpv_render_context`.
pub struct MpvEngine {
    handle: *mut MpvHandle,
    render_ctx: *mut MpvRenderContext,
}

impl MpvEngine {
    pub fn new(hwdec_enabled: bool) -> Option<Self> {
        unsafe {
            // libmpv requiere LC_NUMERIC=C
            let lc_c = CString::new("C").ok()?;
            libc::setlocale(libc::LC_NUMERIC, lc_c.as_ptr());

            let handle = mpv_create();
            if handle.is_null() {
                return None;
            }

            Self::set_opt(handle, "terminal", "yes");
            Self::set_opt(handle, "msg-level", "all=info");
            Self::set_opt(handle, "vo", "libmpv");
            Self::set_opt(handle, "ao", "pipewire,pulse,alsa");
            Self::set_opt(handle, "hwdec", if hwdec_enabled { "auto" } else { "no" });
            Self::set_opt(handle, "keep-open", "yes");
            Self::set_opt(handle, "osc", "no");
            Self::set_opt(handle, "load-console", "no");
            Self::set_opt(handle, "load-scripts", "yes");
            Self::set_opt(handle, "ytdl", "yes");
            Self::set_opt(
                handle,
                "ytdl-format",
                "bestvideo+bestaudio[language~=es]/bestvideo+bestaudio/best",
            );
            Self::set_opt(
                handle,
                "ytdl-raw-options",
                "write-auto-sub=,write-sub=,sub-lang=es,sub-langs=all",
            );
            Self::set_opt(handle, "alang", "es,es-US,es-419,es-ES,spa,Spanish,en");
            Self::set_opt(handle, "slang", "es,es-US,es-419,es-ES,spa,Spanish,en");
            Self::set_opt(handle, "volume", "100");
            Self::set_opt(handle, "mute", "no");
            Self::set_opt(handle, "audio-client-name", "Cine Rust");
            Self::set_opt(handle, "volume-max", "150");
            Self::set_opt(handle, "sub-auto", "fuzzy");
            Self::set_opt(
                handle,
                "sub-file-paths",
                "sub:subs:subtitles:Sub:Subs:Subtitles:srt:srts:Srt:Srts",
            );
            Self::set_opt(handle, "sub-border-size", "2");
            Self::set_opt(handle, "sub-shadow-offset", "0.6");
            Self::set_opt(handle, "sub-border-color", "#B6000000");
            Self::set_opt(handle, "sub-shadow-color", "#97000000");
            Self::set_opt(handle, "sub-color", "#ebebeb");
            Self::set_opt(handle, "sub-use-margins", "no");
            Self::set_opt(handle, "sub-font", "Adwaita Sans SemiBold");

            // Tipografía y estilo visual OSD idéntico a la interfaz GTK4/Libadwaita
            Self::set_opt(handle, "osd-font", "Adwaita Sans");
            Self::set_opt(handle, "osd-bold", "yes");
            Self::set_opt(handle, "osd-bar", "no");
            Self::set_opt(handle, "osd-blur", "1");
            Self::set_opt(handle, "osd-border-size", "1.5");
            Self::set_opt(handle, "osd-shadow-offset", "0.6");
            Self::set_opt(handle, "osd-border-color", "#BE000000");
            Self::set_opt(handle, "osd-shadow-color", "#1B000000");
            Self::set_opt(handle, "osd-margin-x", "66");
            Self::set_opt(handle, "osd-margin-y", "66");
            Self::set_opt(
                handle,
                "script-opts",
                "stats-font=Adwaita Sans,stats-font_mono=Adwaita Sans",
            );

            if mpv_initialize(handle) < 0 {
                mpv_terminate_destroy(handle);
                return None;
            }

            Some(Self {
                handle,
                render_ctx: ptr::null_mut(),
            })
        }
    }

    unsafe fn set_opt(handle: *mut MpvHandle, key: &str, val: &str) {
        if let (Ok(k), Ok(v)) = (CString::new(key), CString::new(val)) {
            unsafe {
                mpv_set_option_string(handle, k.as_ptr(), v.as_ptr());
            }
        }
    }

    pub fn init_gl_context(
        &mut self,
        update_cb: MpvRenderUpdateFn,
        cb_ctx: *mut c_void,
    ) -> bool {
        if !self.render_ctx.is_null() {
            return true;
        }
        unsafe {
            let api_opengl = CString::new("opengl").unwrap();
            let mut init_params = MpvOpenGlInitParams {
                get_proc_address: Some(get_proc_address_trampoline),
                get_proc_address_ctx: ptr::null_mut(),
            };

            let mut params = [
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_API_TYPE,
                    data: api_opengl.as_ptr() as *mut c_void,
                },
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,
                    data: (&mut init_params as *mut MpvOpenGlInitParams).cast(),
                },
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_INVALID,
                    data: ptr::null_mut(),
                },
            ];

            let mut ctx: *mut MpvRenderContext = ptr::null_mut();
            let res = mpv_render_context_create(&mut ctx, self.handle, params.as_mut_ptr());
            if res < 0 || ctx.is_null() {
                eprintln!("[CineRust] mpv_render_context_create failed: {res}");
                return false;
            }
            eprintln!("[CineRust] mpv_render_context_create OK!");

            mpv_render_context_set_update_callback(ctx, Some(update_cb), cb_ctx);
            self.render_ctx = ctx;
            true
        }
    }

    pub fn render_frame(&self, width: i32, height: i32) {
        if self.render_ctx.is_null() {
            return;
        }
        unsafe {
            let mut fbo_id: c_int = 0;
            glGetIntegerv(GL_FRAMEBUFFER_BINDING, &mut fbo_id);

            let mut fbo = MpvOpenGlFbo {
                fbo: fbo_id,
                w: width.max(1),
                h: height.max(1),
                internal_format: 0,
            };
            let mut flip_y: c_int = 1;

            let mut params = [
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_OPENGL_FBO,
                    data: (&mut fbo as *mut MpvOpenGlFbo).cast(),
                },
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_FLIP_Y,
                    data: (&mut flip_y as *mut c_int).cast(),
                },
                MpvRenderParam {
                    type_: MPV_RENDER_PARAM_INVALID,
                    data: ptr::null_mut(),
                },
            ];

            mpv_render_context_render(self.render_ctx, params.as_mut_ptr());
        }
    }

    pub fn command(&self, args: &[&str]) {
        if self.handle.is_null() {
            return;
        }
        let c_strings: Vec<CString> = args
            .iter()
            .filter_map(|s| CString::new(*s).ok())
            .collect();
        let mut ptrs: Vec<*const c_char> = c_strings.iter().map(|s| s.as_ptr()).collect();
        ptrs.push(ptr::null());
        unsafe {
            mpv_command_async(self.handle, 0, ptrs.as_ptr());
        }
    }

    pub fn command_sync(&self, args: &[&str]) {
        if self.handle.is_null() {
            return;
        }
        let c_strings: Vec<CString> = args
            .iter()
            .filter_map(|s| CString::new(*s).ok())
            .collect();
        let mut ptrs: Vec<*const c_char> = c_strings.iter().map(|s| s.as_ptr()).collect();
        ptrs.push(ptr::null());
        unsafe {
            mpv_command(self.handle, ptrs.as_ptr());
        }
    }

    pub fn set_property_str(&self, name: &str, value: &str) {
        if self.handle.is_null() {
            return;
        }
        unsafe {
            Self::set_opt(self.handle, name, value);
        }
    }

    pub fn get_property_str(&self, name: &str) -> Option<String> {
        if self.handle.is_null() {
            return None;
        }
        let c_name = CString::new(name).ok()?;
        unsafe {
            let raw = mpv_get_property_string(self.handle, c_name.as_ptr());
            if raw.is_null() {
                return None;
            }
            let val_str = CStr::from_ptr(raw).to_string_lossy().into_owned();
            mpv_free(raw.cast());
            if val_str.is_empty() {
                None
            } else {
                Some(val_str)
            }
        }
    }

    pub fn get_property_f64(&self, name: &str) -> Option<f64> {
        if self.handle.is_null() {
            return None;
        }
        let c_name = CString::new(name).ok()?;
        unsafe {
            let raw = mpv_get_property_string(self.handle, c_name.as_ptr());
            if raw.is_null() {
                return None;
            }
            let val_str = CStr::from_ptr(raw).to_string_lossy().into_owned();
            mpv_free(raw.cast());
            val_str.parse::<f64>().ok()
        }
    }

    pub fn get_property_bool(&self, name: &str) -> Option<bool> {
        if self.handle.is_null() {
            return None;
        }
        let c_name = CString::new(name).ok()?;
        unsafe {
            let raw = mpv_get_property_string(self.handle, c_name.as_ptr());
            if raw.is_null() {
                return None;
            }
            let val_str = CStr::from_ptr(raw).to_string_lossy().into_owned();
            mpv_free(raw.cast());
            Some(val_str == "yes" || val_str == "true")
        }
    }

    /// Libera el contexto OpenGL primero y luego la instancia de libmpv de forma segura.
    pub fn shutdown(&mut self) {
        unsafe {
            if !self.render_ctx.is_null() {
                mpv_render_context_set_update_callback(self.render_ctx, None, ptr::null_mut());
                mpv_render_context_free(self.render_ctx);
                self.render_ctx = ptr::null_mut();
            }
            if !self.handle.is_null() {
                mpv_terminate_destroy(self.handle);
                self.handle = ptr::null_mut();
            }
        }
    }
}

impl Drop for MpvEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}
