//! An OpenGL ES 2.0 context on an Android window, and the GPU renderer built on it.

use khronos_egl as egl;
use lume_core::render::Gles2Backend;
use std::ffi::{c_void, CString};

/// EGL display, context and window surface. Dropping it releases everything.
pub struct Egl {
    instance: egl::DynamicInstance<egl::EGL1_4>,
    display: egl::Display,
    context: egl::Context,
    surface: egl::Surface,
    /// `EGL_EXT_buffer_age`: the back buffer's old content can be kept and only a part redrawn.
    buffer_age: bool,
    /// `eglSetDamageRegionKHR` (`EGL_KHR_partial_update`), which spares the GPU the rest.
    set_damage: Option<SetDamageRegion>,
}

type SetDamageRegion = unsafe extern "C" fn(*mut c_void, *mut c_void, *mut i32, i32) -> u32;
const BUFFER_AGE_EXT: i32 = 0x313D;

impl Egl {
    /// Creates a context for `window` (an `ANativeWindow`), without depth or stencil buffers.
    pub fn new(window: *mut c_void) -> Result<Egl, String> {
        // SAFETY: loading the system EGL library.
        let instance = unsafe {
            egl::DynamicInstance::<egl::EGL1_4>::load_required_from_filename("libEGL.so")
        }
        .map_err(|e| format!("cannot load libEGL: {e}"))?;
        // SAFETY: the default display is always valid to request.
        let display =
            unsafe { instance.get_display(egl::DEFAULT_DISPLAY) }.ok_or("no EGL display")?;
        instance
            .initialize(display)
            .map_err(|e| format!("eglInitialize: {e}"))?;

        let attributes = [
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES2_BIT,
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::DEPTH_SIZE,
            0,
            egl::STENCIL_SIZE,
            0,
            egl::NONE,
        ];
        let config = instance
            .choose_first_config(display, &attributes)
            .map_err(|e| format!("eglChooseConfig: {e}"))?
            .ok_or("no RGBA8888 OpenGL ES 2.0 config")?;
        // SAFETY: `window` is a valid ANativeWindow for the lifetime of the surface.
        let surface =
            unsafe { instance.create_window_surface(display, config, window.cast(), None) }
                .map_err(|e| format!("eglCreateWindowSurface: {e}"))?;
        let context = instance
            .create_context(
                display,
                config,
                None,
                &[egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE],
            )
            .map_err(|e| format!("eglCreateContext: {e}"))?;
        instance
            .make_current(display, Some(surface), Some(surface), Some(context))
            .map_err(|e| format!("eglMakeCurrent: {e}"))?;
        let extensions = instance
            .query_string(Some(display), egl::EXTENSIONS)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let has = |name: &str| extensions.split(' ').any(|e| e == name);
        let buffer_age = has("EGL_EXT_buffer_age");
        let set_damage = (buffer_age && has("EGL_KHR_partial_update"))
            .then(|| instance.get_proc_address("eglSetDamageRegionKHR"))
            .flatten()
            // SAFETY: the signature EGL_KHR_partial_update gives this function.
            .map(|f| unsafe { std::mem::transmute::<extern "system" fn(), SetDamageRegion>(f) });
        log::info!(
            "partial redraw: buffer age {buffer_age}, damage region {}",
            set_damage.is_some()
        );
        Ok(Egl {
            instance,
            display,
            context,
            surface,
            buffer_age,
            set_damage,
        })
    }

    /// A `glow` context for the current EGL context.
    pub fn glow(&self) -> glow::Context {
        // GLES2 functions are normally resolved by eglGetProcAddress; fall back to the library.
        // SAFETY: the library name is a valid C string, and the symbol names come from glow.
        let gles = unsafe {
            libc::dlopen(
                CString::new("libGLESv2.so").unwrap().as_ptr(),
                libc::RTLD_NOW,
            )
        };
        unsafe {
            glow::Context::from_loader_function(|name| {
                if let Some(function) = self.instance.get_proc_address(name) {
                    return function as *const c_void;
                }
                match CString::new(name) {
                    Ok(symbol) if !gles.is_null() => {
                        libc::dlsym(gles, symbol.as_ptr()).cast_const()
                    }
                    _ => std::ptr::null(),
                }
            })
        }
    }

    /// How many frames ago the back buffer was the screen's content, or 0 when its content is
    /// unknown (new buffer, or no `EGL_EXT_buffer_age`) and the frame must be drawn whole.
    pub fn buffer_age(&self) -> usize {
        if !self.buffer_age {
            return 0;
        }
        self.instance
            .query_surface(self.display, self.surface, BUFFER_AGE_EXT)
            .map_or(0, |age| age.max(0) as usize)
    }

    /// Tells the driver the only part of the buffer this frame draws (a GL scissor box). Must
    /// follow [`Egl::buffer_age`] in the frame.
    pub fn set_damage(&self, [x, y, w, h]: [i32; 4]) {
        if let Some(set_damage) = self.set_damage {
            let mut rect = [x, y, w, h];
            // SAFETY: the display and surface are alive, and `rect` is one rect of four ints.
            unsafe {
                set_damage(
                    self.display.as_ptr(),
                    self.surface.as_ptr(),
                    rect.as_mut_ptr(),
                    1,
                )
            };
        }
    }

    pub fn swap_buffers(&self) -> Result<(), String> {
        self.instance
            .swap_buffers(self.display, self.surface)
            .map_err(|e| format!("eglSwapBuffers: {e}"))
    }
}

impl Drop for Egl {
    fn drop(&mut self) {
        let _ = self.instance.make_current(self.display, None, None, None);
        let _ = self.instance.destroy_surface(self.display, self.surface);
        let _ = self.instance.destroy_context(self.display, self.context);
        let _ = self.instance.terminate(self.display);
    }
}

/// The window's EGL context and the renderer that draws on it. Exists only while the window does.
pub struct Graphics {
    // Declaration order matters for drop: the renderer's objects go before the context.
    backend: Option<Gles2Backend>,
    pub egl: Egl,
}

impl Graphics {
    pub fn new(window: *mut c_void) -> Result<Graphics, String> {
        let egl = Egl::new(window)?;
        let backend = Gles2Backend::new(egl.glow())?;
        Ok(Graphics {
            backend: Some(backend),
            egl,
        })
    }

    pub fn backend(&mut self) -> &mut Gles2Backend {
        self.backend
            .as_mut()
            .expect("the backend exists until the graphics are dropped")
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        // The GL objects must be deleted while their context is current.
        if let Some(backend) = self.backend.take() {
            backend.destroy();
        }
    }
}
