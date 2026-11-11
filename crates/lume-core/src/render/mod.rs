//! Rendering: a backend-independent display list, batching and the backend trait.

mod batch;
mod geometry;
mod gles2;
mod list;
mod shaders;

pub use batch::{
    Batch, Frame, Program, RectVertex, TexVertex, Texture, FULL_DAMAGE, MAX_QUADS_PER_BATCH,
    QUAD_INDICES,
};
pub use geometry::Rect;
pub use gles2::Gles2Backend;
pub use list::{Cmd, DisplayList, RectCmd, RectKind, TexturedCmd};

use crate::theme::Color;

/// Where the logical canvas lands in the device surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    /// Surface size in device pixels.
    pub width: u32,
    pub height: u32,
    /// Device pixels per logical unit.
    pub scale: f32,
    /// Device-pixel offset of the logical origin (centers the canvas on the screen).
    pub offset_x: f32,
    pub offset_y: f32,
    /// Width of the antialiasing ramp of every shape and clipped image, in device pixels: the
    /// theme's `--edge-softness`.
    pub edge_softness: f32,
}

/// What a graphics backend must provide. `lume-core` produces [`Frame`]s;
/// the backend (OpenGL ES 2.0) turns them into pixels.
pub trait Backend {
    /// Draws `frame` on top of a surface cleared to `clear`.
    fn render(&mut self, viewport: &Viewport, clear: Color, frame: &Frame);
}
