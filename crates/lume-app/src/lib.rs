//! Android shell of lume: NativeActivity, EGL, JNI and the event loop.
//!
//! Everything that is not tied to Android lives in `lume-core` (the launcher itself and the
//! [`lume_core::shell::Shell`] that owns its lifecycle); this crate only connects it to the
//! platform: the window and OpenGL ES, the keys, and the Java bridge.

#[cfg(target_os = "android")]
mod android;
