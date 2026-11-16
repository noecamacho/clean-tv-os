//! An OpenGL ES context (3.2 where the TV has it, 2.0 otherwise) on an Android window, and the GPU renderer built on it.

use super::vulkan::VkRenderer;
use khronos_egl as egl;
use lume_core::render::{Frame, Gles2Backend, Viewport};
use lume_core::text::TextEngine;
use lume_core::theme::Color;
use lume_core::wallpaper::WallpaperId;
use std::ffi::{c_void, CString};
use std::path::PathBuf;

/// EGL display, context and window surface. Dropping it releases everything.
pub struct Egl {
    instance: egl::DynamicInstance<egl::EGL1_4>,
    display: egl::Display,
    config: egl::Config,
    context: egl::Context,
    /// `None` while the window is away and the context is kept (see [`Egl::detach`]).
    surface: Option<egl::Surface>,
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

        // The newest OpenGL ES the TV has: 3.2, then 3.0, then the 2.0 every Android TV has. The
        // shaders are GLSL ES 1.00, which every one of them compiles.
        const OPENGL_ES3_BIT: egl::Int = 0x40;
        const CONTEXT_MINOR_VERSION: egl::Int = 0x30FB;
        let mut made = None;
        for (bit, major, minor) in [
            (OPENGL_ES3_BIT, 3, 2),
            (OPENGL_ES3_BIT, 3, 0),
            (egl::OPENGL_ES2_BIT, 2, 0),
        ] {
            let attributes = [
                egl::RENDERABLE_TYPE,
                bit,
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
            let Ok(Some(config)) = instance.choose_first_config(display, &attributes) else {
                continue;
            };
            let version = if minor > 0 {
                vec![
                    egl::CONTEXT_CLIENT_VERSION,
                    major,
                    CONTEXT_MINOR_VERSION,
                    minor,
                    egl::NONE,
                ]
            } else {
                vec![egl::CONTEXT_CLIENT_VERSION, major, egl::NONE]
            };
            if let Ok(context) = instance.create_context(display, config, None, &version) {
                log::info!("OpenGL ES {major}.{minor} context");
                made = Some((config, context));
                break;
            }
        }
        let (config, context) = made.ok_or("no RGBA8888 OpenGL ES config")?;
        // SAFETY: `window` is a valid ANativeWindow for the lifetime of the surface.
        let surface =
            unsafe { instance.create_window_surface(display, config, window.cast(), None) }
                .map_err(|e| format!("eglCreateWindowSurface: {e}"))?;
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
            config,
            context,
            surface: Some(surface),
            buffer_age,
            set_damage,
        })
    }

    /// The window is going away: its surface is destroyed and the context, with every texture
    /// and program, is kept for [`Egl::attach`].
    pub fn detach(&mut self) {
        let _ = self.instance.make_current(self.display, None, None, None);
        if let Some(surface) = self.surface.take() {
            let _ = self.instance.destroy_surface(self.display, surface);
        }
    }

    /// A surface for the new `window`, current with the context that was kept.
    pub fn attach(&mut self, window: *mut c_void) -> Result<(), String> {
        // SAFETY: `window` is a valid ANativeWindow for the lifetime of the surface.
        let surface = unsafe {
            self.instance
                .create_window_surface(self.display, self.config, window.cast(), None)
        }
        .map_err(|e| format!("eglCreateWindowSurface: {e}"))?;
        self.surface = Some(surface);
        self.instance
            .make_current(
                self.display,
                Some(surface),
                Some(surface),
                Some(self.context),
            )
            .map_err(|e| format!("eglMakeCurrent: {e}"))
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
        let Some(surface) = self.surface.filter(|_| self.buffer_age) else {
            return 0;
        };
        self.instance
            .query_surface(self.display, surface, BUFFER_AGE_EXT)
            .map_or(0, |age| age.max(0) as usize)
    }

    /// Tells the driver the only part of the buffer this frame draws (a GL scissor box). Must
    /// follow [`Egl::buffer_age`] in the frame.
    pub fn set_damage(&self, [x, y, w, h]: [i32; 4]) {
        if let (Some(set_damage), Some(surface)) = (self.set_damage, self.surface) {
            let mut rect = [x, y, w, h];
            // SAFETY: the display and surface are alive, and `rect` is one rect of four ints.
            unsafe {
                set_damage(
                    self.display.as_ptr(),
                    surface.as_ptr(),
                    rect.as_mut_ptr(),
                    1,
                )
            };
        }
    }

    pub fn swap_buffers(&self) -> Result<(), String> {
        let surface = self.surface.ok_or("no surface")?;
        self.instance
            .swap_buffers(self.display, surface)
            .map_err(|e| format!("eglSwapBuffers: {e}"))
    }
}

impl Drop for Egl {
    fn drop(&mut self) {
        self.detach();
        let _ = self.instance.destroy_context(self.display, self.context);
        let _ = self.instance.terminate(self.display);
    }
}

/// The renderer of the window: OpenGL ES, or Vulkan 1.3 when asked for (see
/// `renderer_property`). Exists only while the window does.
// One of them exists at a time, on the stack of the loop: boxing the larger buys nothing.
#[allow(clippy::large_enum_variant)]
pub enum Graphics {
    Vulkan(Box<VkRenderer>),
    Gl {
        // Declaration order matters for drop: the renderer's objects go before the context.
        backend: Option<Gles2Backend>,
        egl: Egl,
    },
}

/// The value of the system property `debug.lume.renderer` (`adb shell setprop`): `vulkan` picks
/// the Vulkan renderer, to compare the two on the same TV. Measured on the TCL 55Q5K, it is as
/// fluid as OpenGL ES and slower to start, so OpenGL ES stays the default.
fn renderer_property() -> String {
    let mut value = [0u8; 92];
    // SAFETY: the buffer is PROP_VALUE_MAX bytes, which the call never exceeds.
    let length = unsafe {
        libc::__system_property_get(c"debug.lume.renderer".as_ptr(), value.as_mut_ptr().cast())
    };
    String::from_utf8_lossy(&value[..length.max(0) as usize]).into_owned()
}

/// Forwards a call to whichever renderer the window has.
macro_rules! renderer {
    ($self:ident, $r:ident => $call:expr) => {
        match $self {
            Graphics::Vulkan($r) => $call,
            Graphics::Gl { backend, .. } => {
                let $r = backend.as_mut().expect("the backend exists until drop");
                $call
            }
        }
    };
}

impl Graphics {
    pub fn new(
        window: *mut c_void,
        size: (u32, u32),
        cache_dir: Option<PathBuf>,
    ) -> Result<Graphics, String> {
        if renderer_property() == "vulkan" {
            match VkRenderer::new(window, size, cache_dir) {
                Ok(renderer) => return Ok(Graphics::Vulkan(Box::new(renderer))),
                Err(e) => log::warn!("no Vulkan renderer ({e}): using OpenGL ES"),
            }
        }
        let egl = Egl::new(window)?;
        let mut backend = Gles2Backend::new(egl.glow())?;
        backend.set_render_size(size.0, size.1);
        Ok(Graphics::Gl {
            backend: Some(backend),
            egl,
        })
    }

    /// The window is going away. True when the renderer could be kept without it (OpenGL ES:
    /// the context, programs and textures stay), to come back with [`Graphics::resume`].
    pub fn park(&mut self) -> bool {
        match self {
            Graphics::Vulkan(_) => false,
            Graphics::Gl { egl, .. } => {
                egl.detach();
                true
            }
        }
    }

    /// Draws on the new `window` with what [`Graphics::park`] kept.
    pub fn resume(&mut self, window: *mut c_void, size: (u32, u32)) -> Result<(), String> {
        match self {
            Graphics::Vulkan(_) => Err("the Vulkan renderer is not kept".into()),
            Graphics::Gl { egl, backend } => {
                egl.attach(window)?;
                if let Some(backend) = backend.as_mut() {
                    backend.set_render_size(size.0, size.1);
                }
                Ok(())
            }
        }
    }

    pub fn set_render_size(&mut self, width: u32, height: u32) {
        renderer!(self, r => r.set_render_size(width, height))
    }

    pub fn set_async_wallpaper(&mut self, enabled: bool) {
        renderer!(self, r => r.set_async_wallpaper(enabled))
    }

    pub fn wallpaper_uploading(&mut self) -> bool {
        renderer!(self, r => r.wallpaper_uploading())
    }

    pub fn has_wallpaper(&mut self) -> bool {
        renderer!(self, r => r.has_wallpaper())
    }

    pub fn poll_wallpaper(&mut self, id: WallpaperId) {
        renderer!(self, r => r.poll_wallpaper(id))
    }

    pub fn gpu_strings(&mut self) -> (String, String) {
        renderer!(self, r => r.gpu_strings())
    }

    pub fn ensure_image(&mut self, width: u32, height: u32) {
        renderer!(self, r => r.ensure_image(width, height))
    }

    pub fn update_image(&mut self, region: [u32; 4], rgba: &[u8]) {
        renderer!(self, r => r.update_image(region, rgba))
    }

    pub fn sync_text_atlas(&mut self, engine: &mut TextEngine) {
        renderer!(self, r => r.sync_text_atlas(engine))
    }

    pub fn prepare(&mut self, frame: &Frame) {
        renderer!(self, r => r.prepare(frame))
    }

    pub fn take_content_changed(&mut self) -> bool {
        renderer!(self, r => r.take_content_changed())
    }

    /// How many frames ago the buffer of the next frame was on screen (0: unknown, draw it whole).
    pub fn buffer_age(&mut self) -> usize {
        match self {
            Graphics::Vulkan(renderer) => renderer.buffer_age(),
            Graphics::Gl { egl, .. } => egl.buffer_age(),
        }
    }

    /// Draws `frame` and shows it; with `damage` (an OpenGL scissor box), only that part, over
    /// what the buffer already had. Must follow [`Graphics::buffer_age`] in the frame.
    pub fn present(
        &mut self,
        viewport: &Viewport,
        clear: Color,
        frame: &Frame,
        damage: Option<[i32; 4]>,
    ) -> Result<(), String> {
        match self {
            Graphics::Vulkan(renderer) => renderer.present(viewport, clear, frame, damage),
            Graphics::Gl { backend, egl } => {
                if let Some(damage) = damage {
                    egl.set_damage(damage);
                }
                backend
                    .as_mut()
                    .expect("the backend exists until drop")
                    .render_damaged(viewport, clear, frame, damage);
                egl.swap_buffers()
            }
        }
    }
}

impl Drop for Graphics {
    fn drop(&mut self) {
        // The GL objects must be deleted while their context is current, which it is not for a
        // renderer dropped while parked: then the context takes them with it.
        if let Graphics::Gl { backend, egl } = self {
            if egl.surface.is_none() {
                std::mem::forget(backend.take());
                return;
            }
            if let Some(backend) = backend.take() {
                backend.destroy();
            }
        }
    }
}
