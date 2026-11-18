//! The dozen SDL2 calls HOME needs, as raw FFI.
//!
//! No `sdl2` crate: the TV's SDL-webOS delivers scancodes (Back 482, Home 384, Exit 505)
//! outside that crate's closed `Scancode` enum, which can abort on `transmute`. Reading the
//! event buffer by offset sidesteps the problem and saves the dependency.

use std::ffi::{c_char, c_int, c_void, CStr, CString};

pub const INIT_VIDEO: u32 = 0x0000_0020;

// SDL_GLattr values (SDL_video.h).
pub const GL_RED_SIZE: c_int = 0;
pub const GL_GREEN_SIZE: c_int = 1;
pub const GL_BLUE_SIZE: c_int = 2;
pub const GL_ALPHA_SIZE: c_int = 3;
pub const GL_DOUBLEBUFFER: c_int = 5;
pub const GL_DEPTH_SIZE: c_int = 6;
pub const GL_STENCIL_SIZE: c_int = 7;
pub const GL_CONTEXT_MAJOR_VERSION: c_int = 17;
pub const GL_CONTEXT_MINOR_VERSION: c_int = 18;
pub const GL_CONTEXT_PROFILE_MASK: c_int = 21;
pub const GL_CONTEXT_PROFILE_ES: c_int = 0x0004;

pub const WINDOW_OPENGL: u32 = 0x0000_0002;
pub const WINDOW_FULLSCREEN_DESKTOP: u32 = 0x0000_1001;
pub const WINDOWPOS_UNDEFINED: c_int = 0x1FFF_0000;

// SDL_EventType.
pub const QUIT: u32 = 0x100;
pub const WINDOWEVENT: u32 = 0x200;
pub const KEYDOWN: u32 = 0x300;
pub const KEYUP: u32 = 0x301;

// SDL_WindowEventID.
pub const WINDOWEVENT_SHOWN: u8 = 1;
pub const WINDOWEVENT_HIDDEN: u8 = 2;
pub const WINDOWEVENT_EXPOSED: u8 = 3;
pub const WINDOWEVENT_RESTORED: u8 = 9;
pub const WINDOWEVENT_FOCUS_GAINED: u8 = 12;
pub const WINDOWEVENT_FOCUS_LOST: u8 = 13;
pub const WINDOWEVENT_CLOSE: u8 = 14;

// Scancodes: the standard USB ones, plus SDL-webOS extensions (see Luma's sdl_events.rs).
pub const SCANCODE_RETURN: i32 = 40;
pub const SCANCODE_ESCAPE: i32 = 41;
pub const SCANCODE_RIGHT: i32 = 79;
pub const SCANCODE_LEFT: i32 = 80;
pub const SCANCODE_DOWN: i32 = 81;
pub const SCANCODE_UP: i32 = 82;
pub const SCANCODE_KP_ENTER: i32 = 88;
pub const SCANCODE_WEBOS_HOME: i32 = 384;
pub const SCANCODE_WEBOS_BACK: i32 = 482;
pub const SCANCODE_WEBOS_EXIT: i32 = 505;

/// An `SDL_Event` (56 bytes in SDL2), read by field offset.
#[repr(C, align(8))]
pub struct Event(pub [u8; 64]);

impl Event {
    pub fn new() -> Event {
        Event([0; 64])
    }

    pub fn kind(&self) -> u32 {
        u32::from_ne_bytes(self.0[0..4].try_into().expect("4 bytes"))
    }

    /// `SDL_WindowEvent.event` (the window event id).
    pub fn window_event(&self) -> u8 {
        self.0[12]
    }

    /// `SDL_KeyboardEvent.keysym.scancode`.
    pub fn key_scancode(&self) -> i32 {
        i32::from_ne_bytes(self.0[16..20].try_into().expect("4 bytes"))
    }

    /// `SDL_KeyboardEvent.repeat`.
    pub fn key_repeat(&self) -> bool {
        self.0[13] != 0
    }
}

#[link(name = "SDL2")]
extern "C" {
    fn SDL_Init(flags: u32) -> c_int;
    fn SDL_GetError() -> *const c_char;
    fn SDL_GL_SetAttribute(attr: c_int, value: c_int) -> c_int;
    fn SDL_CreateWindow(
        title: *const c_char,
        x: c_int,
        y: c_int,
        w: c_int,
        h: c_int,
        flags: u32,
    ) -> *mut c_void;
    fn SDL_GL_CreateContext(window: *mut c_void) -> *mut c_void;
    fn SDL_GL_SetSwapInterval(interval: c_int) -> c_int;
    fn SDL_GL_SwapWindow(window: *mut c_void);
    fn SDL_GL_GetProcAddress(name: *const c_char) -> *mut c_void;
    fn SDL_GL_GetDrawableSize(window: *mut c_void, w: *mut c_int, h: *mut c_int);
    fn SDL_WaitEventTimeout(event: *mut Event, timeout: c_int) -> c_int;
    fn SDL_PollEvent(event: *mut Event) -> c_int;
    fn SDL_GetVideoDriver(index: c_int) -> *const c_char;
    fn SDL_GetCurrentVideoDriver() -> *const c_char;
}

fn error() -> String {
    // SAFETY: SDL_GetError always returns a valid NUL-terminated string.
    unsafe { CStr::from_ptr(SDL_GetError()) }
        .to_string_lossy()
        .into_owned()
}

/// The SDL window with a current GLES2 context.
pub struct Window {
    handle: *mut c_void,
}

impl Window {
    /// Initializes SDL video and opens a fullscreen window with a GLES 2.0 context.
    pub fn open(title: &str, width: u32, height: u32) -> Result<Window, String> {
        // SAFETY: plain SDL calls from the main thread, checked for errors.
        unsafe {
            if SDL_Init(INIT_VIDEO) != 0 {
                let driver = CStr::from_ptr(SDL_GetVideoDriver(0)).to_string_lossy();
                return Err(format!("SDL_Init: {} (driver 0: {driver})", error()));
            }
            log::info!(
                "SDL video driver: {}",
                CStr::from_ptr(SDL_GetCurrentVideoDriver()).to_string_lossy()
            );
            for (attr, value) in [
                (GL_CONTEXT_PROFILE_MASK, GL_CONTEXT_PROFILE_ES),
                (GL_CONTEXT_MAJOR_VERSION, 2),
                (GL_CONTEXT_MINOR_VERSION, 0),
                (GL_RED_SIZE, 8),
                (GL_GREEN_SIZE, 8),
                (GL_BLUE_SIZE, 8),
                (GL_ALPHA_SIZE, 8),
                (GL_DEPTH_SIZE, 0),
                (GL_STENCIL_SIZE, 0),
                (GL_DOUBLEBUFFER, 1),
            ] {
                SDL_GL_SetAttribute(attr, value);
            }
            let title = CString::new(title).map_err(|_| "window title")?;
            let handle = SDL_CreateWindow(
                title.as_ptr(),
                WINDOWPOS_UNDEFINED,
                WINDOWPOS_UNDEFINED,
                width as c_int,
                height as c_int,
                WINDOW_OPENGL | WINDOW_FULLSCREEN_DESKTOP,
            );
            if handle.is_null() {
                return Err(format!("SDL_CreateWindow: {}", error()));
            }
            if SDL_GL_CreateContext(handle).is_null() {
                return Err(format!("SDL_GL_CreateContext: {}", error()));
            }
            if SDL_GL_SetSwapInterval(1) != 0 {
                log::warn!("vsync: {}", error());
            }
            Ok(Window { handle })
        }
    }

    /// The size of the GL framebuffer in pixels.
    pub fn drawable_size(&self) -> (u32, u32) {
        let (mut w, mut h) = (0, 0);
        // SAFETY: the window is alive; the out-pointers are valid.
        unsafe { SDL_GL_GetDrawableSize(self.handle, &mut w, &mut h) };
        (w.max(1) as u32, h.max(1) as u32)
    }

    /// A `glow` context over SDL's GL loader.
    pub fn glow(&self) -> glow::Context {
        // SAFETY: the GLES context created in `open` is current on this thread.
        unsafe {
            glow::Context::from_loader_function(|name| match CString::new(name) {
                Ok(name) => SDL_GL_GetProcAddress(name.as_ptr()).cast_const(),
                Err(_) => std::ptr::null(),
            })
        }
    }

    pub fn swap(&self) {
        // SAFETY: the window and its context are alive and current.
        unsafe { SDL_GL_SwapWindow(self.handle) };
    }

    /// Waits up to `timeout` for an event. `None` waits for the next event without a limit.
    pub fn wait_event(&self, timeout: Option<std::time::Duration>) -> Option<Event> {
        let mut event = Event::new();
        let got = match timeout {
            // SAFETY: `event` is a valid buffer; SDL fills it only on success.
            Some(t) => unsafe {
                SDL_WaitEventTimeout(&mut event, t.as_millis().min(i32::MAX as u128) as c_int)
            },
            None => unsafe { SDL_WaitEventTimeout(&mut event, -1) },
        };
        (got == 1).then_some(event)
    }

    /// The next already-queued event, without waiting.
    pub fn poll_event(&self) -> Option<Event> {
        let mut event = Event::new();
        // SAFETY: `event` is a valid buffer; SDL fills it only on success.
        (unsafe { SDL_PollEvent(&mut event) } == 1).then_some(event)
    }
}
