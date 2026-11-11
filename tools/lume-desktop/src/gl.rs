//! A headless OpenGL ES 2.0 context for tests and screenshots.
//!
//! It loads the system `libEGL` at runtime (Mesa on Linux), so nothing needs to be
//! installed at build time. Rendering goes to an off-screen pbuffer.

use khronos_egl as egl;

pub struct HeadlessGl {
    egl: egl::DynamicInstance<egl::EGL1_4>,
    display: egl::Display,
    surface: egl::Surface,
    context: egl::Context,
    pub width: u32,
    pub height: u32,
}

impl HeadlessGl {
    /// Creates a context with a `width` x `height` off-screen surface.
    pub fn new(width: u32, height: u32) -> Result<HeadlessGl, String> {
        // SAFETY: loading the system EGL library; its initializers are trusted.
        let egl = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required() }
            .map_err(|e| format!("cannot load libEGL: {e}"))?;
        let display = unsafe { egl.get_display(egl::DEFAULT_DISPLAY) }.ok_or("no EGL display")?;
        egl.initialize(display)
            .map_err(|e| format!("eglInitialize failed: {e}"))?;

        let config_attributes = [
            egl::SURFACE_TYPE,
            egl::PBUFFER_BIT,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES2_BIT,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::NONE,
        ];
        let config = egl
            .choose_first_config(display, &config_attributes)
            .map_err(|e| format!("eglChooseConfig failed: {e}"))?
            .ok_or("no suitable EGL config (OpenGL ES 2.0 pbuffer)")?;

        let surface_attributes = [
            egl::WIDTH,
            width as i32,
            egl::HEIGHT,
            height as i32,
            egl::NONE,
        ];
        let surface = egl
            .create_pbuffer_surface(display, config, &surface_attributes)
            .map_err(|e| format!("cannot create the pbuffer: {e}"))?;

        egl.bind_api(egl::OPENGL_ES_API)
            .map_err(|e| format!("cannot bind OpenGL ES: {e}"))?;
        let context = egl
            .create_context(
                display,
                config,
                None,
                &[egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE],
            )
            .map_err(|e| format!("cannot create the GLES2 context: {e}"))?;
        egl.make_current(display, Some(surface), Some(surface), Some(context))
            .map_err(|e| format!("eglMakeCurrent failed: {e}"))?;
        Ok(HeadlessGl {
            egl,
            display,
            surface,
            context,
            width,
            height,
        })
    }

    /// A `glow` context for the current EGL context.
    pub fn glow(&self) -> glow::Context {
        // SAFETY: the EGL context created in `new` is current on this thread.
        unsafe {
            glow::Context::from_loader_function(|name| {
                self.egl
                    .get_proc_address(name)
                    .map_or(std::ptr::null(), |f| f as *const _)
            })
        }
    }
}

impl Drop for HeadlessGl {
    fn drop(&mut self) {
        let _ = self.egl.make_current(self.display, None, None, None);
        let _ = self.egl.destroy_context(self.display, self.context);
        let _ = self.egl.destroy_surface(self.display, self.surface);
        let _ = self.egl.terminate(self.display);
    }
}
