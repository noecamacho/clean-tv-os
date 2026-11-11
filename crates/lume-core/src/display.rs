//! Mapping between the logical 1280x720 canvas and the device surface.

use crate::render::{Rect, Viewport};

/// Size of the logical canvas every screen is laid out on.
pub const CANVAS_WIDTH: f32 = 1280.0;
pub const CANVAS_HEIGHT: f32 = 720.0;

/// The largest buffer HOME renders to on Android TV; the compositor upscales larger surfaces.
pub const MAX_RENDER_WIDTH: u32 = 1920;
pub const MAX_RENDER_HEIGHT: u32 = 1080;

/// Interactive content and text stay at least this far from the canvas edges (logical units),
/// so overscan cannot cut them.
pub const SAFE_MARGIN_X: f32 = 32.0;
pub const SAFE_MARGIN_Y: f32 = 18.0;

/// Size of the buffer to render for a screen of `width` x `height`: the screen size, limited
/// to 1920x1080 with the aspect ratio kept.
pub fn render_size(width: u32, height: u32) -> (u32, u32) {
    if width <= MAX_RENDER_WIDTH && height <= MAX_RENDER_HEIGHT {
        return (width, height);
    }
    let factor =
        (MAX_RENDER_WIDTH as f32 / width as f32).min(MAX_RENDER_HEIGHT as f32 / height as f32);
    (
        ((width as f32 * factor).floor() as u32).max(1),
        ((height as f32 * factor).floor() as u32).max(1),
    )
}

/// How the canvas fits on a surface: uniform scale, centered.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Display {
    /// Surface size in device pixels.
    pub width: u32,
    pub height: u32,
    /// Device pixels per logical unit: `min(width / 1280, height / 720)`.
    pub scale: f32,
    /// Device-pixel offset of the logical origin. Whole pixels, so edges stay crisp.
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Display {
    pub fn new(width: u32, height: u32) -> Display {
        let scale = (width as f32 / CANVAS_WIDTH).min(height as f32 / CANVAS_HEIGHT);
        Display {
            width,
            height,
            scale,
            offset_x: ((width as f32 - CANVAS_WIDTH * scale) / 2.0).round(),
            offset_y: ((height as f32 - CANVAS_HEIGHT * scale) / 2.0).round(),
        }
    }

    pub fn viewport(&self) -> Viewport {
        Viewport {
            width: self.width,
            height: self.height,
            scale: self.scale,
            offset_x: self.offset_x,
            offset_y: self.offset_y,
            edge_softness: crate::theme::vars::EDGE_SOFTNESS,
        }
    }

    /// The logical area that covers the whole surface. On a 16:9 screen it is the canvas;
    /// on other shapes it extends past the canvas and only backgrounds should fill it.
    pub fn visible_rect(&self) -> Rect {
        Rect::new(
            -self.offset_x / self.scale,
            -self.offset_y / self.scale,
            self.width as f32 / self.scale,
            self.height as f32 / self.scale,
        )
    }

    /// The part of the canvas where interactive content and text may go.
    pub fn safe_area(&self) -> Rect {
        Rect::new(
            SAFE_MARGIN_X,
            SAFE_MARGIN_Y,
            CANVAS_WIDTH - 2.0 * SAFE_MARGIN_X,
            CANVAS_HEIGHT - 2.0 * SAFE_MARGIN_Y,
        )
    }

    /// Rounds a logical coordinate to the nearest device pixel.
    pub fn snap(&self, value: f32) -> f32 {
        (value * self.scale).round() / self.scale
    }

    /// Snaps a rectangle's edges (not its size) to device pixels, so neighbors never leave gaps.
    pub fn snap_rect(&self, rect: Rect) -> Rect {
        let (left, top) = (self.snap(rect.x), self.snap(rect.y));
        Rect::new(
            left,
            top,
            self.snap(rect.right()) - left,
            self.snap(rect.bottom()) - top,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hd_is_one_to_one() {
        let d = Display::new(1280, 720);
        assert_eq!((d.scale, d.offset_x, d.offset_y), (1.0, 0.0, 0.0));
        assert_eq!(d.visible_rect(), Rect::new(0.0, 0.0, 1280.0, 720.0));
    }

    #[test]
    fn full_hd_is_one_and_a_half() {
        let d = Display::new(1920, 1080);
        assert_eq!((d.scale, d.offset_x, d.offset_y), (1.5, 0.0, 0.0));
        assert_eq!(d.viewport().scale, 1.5);
    }

    #[test]
    fn a_taller_screen_is_centered_vertically() {
        // 16:10: width limits the scale.
        let d = Display::new(1920, 1200);
        assert_eq!(d.scale, 1.5);
        assert_eq!((d.offset_x, d.offset_y), (0.0, 60.0));
        let visible = d.visible_rect();
        assert_eq!(
            (visible.x, visible.y, visible.w, visible.h),
            (0.0, -40.0, 1280.0, 800.0)
        );
    }

    #[test]
    fn a_wider_screen_is_centered_horizontally() {
        // 21:9: height limits the scale.
        let d = Display::new(2560, 1080);
        assert_eq!(d.scale, 1.5);
        assert_eq!((d.offset_x, d.offset_y), (320.0, 0.0));
        let visible = d.visible_rect();
        assert!((visible.x + 320.0 / 1.5).abs() < 1e-3);
        assert!((visible.w - 2560.0 / 1.5).abs() < 1e-3);
    }

    #[test]
    fn odd_sizes_use_a_fractional_scale_and_whole_pixel_offsets() {
        let d = Display::new(1366, 768);
        assert!((d.scale - 1.0672).abs() < 1e-3);
        assert_eq!(d.offset_x, 0.0);
        assert_eq!(d.offset_y, 0.0);
        let d = Display::new(1001, 700);
        assert!(d.scale < 1.0);
        assert_eq!(d.offset_x.fract(), 0.0);
        assert_eq!(d.offset_y.fract(), 0.0);
    }

    #[test]
    fn the_canvas_always_fits_in_the_surface() {
        for (w, h) in [
            (1280, 720),
            (1920, 1080),
            (1920, 1200),
            (2560, 1080),
            (1366, 768),
            (1001, 700),
            (800, 480),
        ] {
            let d = Display::new(w, h);
            let right = d.offset_x + CANVAS_WIDTH * d.scale;
            let bottom = d.offset_y + CANVAS_HEIGHT * d.scale;
            assert!(d.offset_x >= 0.0 && d.offset_y >= 0.0, "{w}x{h}");
            assert!(
                right <= w as f32 + 1.0 && bottom <= h as f32 + 1.0,
                "{w}x{h}"
            );
        }
    }

    #[test]
    fn render_size_caps_at_full_hd() {
        assert_eq!(render_size(3840, 2160), (1920, 1080));
        assert_eq!(render_size(1280, 720), (1280, 720));
        assert_eq!(render_size(1920, 1080), (1920, 1080));
        // Other shapes keep their aspect ratio.
        assert_eq!(render_size(4096, 2160), (1920, 1012));
        assert_eq!(render_size(2560, 1080), (1920, 810));
    }

    #[test]
    fn the_safe_area_respects_the_margins() {
        let safe = Display::new(1280, 720).safe_area();
        assert_eq!((safe.x, safe.y), (32.0, 18.0));
        assert_eq!((safe.right(), safe.bottom()), (1248.0, 702.0));
    }

    #[test]
    fn snapping_lands_on_device_pixels() {
        let d = Display::new(1920, 1080);
        for v in [0.3, 10.49, 33.333, 1279.9] {
            let device = d.snap(v) * d.scale;
            assert!((device - device.round()).abs() < 1e-3);
        }
        let r = d.snap_rect(Rect::new(10.1, 20.2, 99.9, 50.3));
        for v in [r.x, r.y, r.right(), r.bottom()] {
            let device = v * d.scale;
            assert!((device - device.round()).abs() < 1e-3);
        }
    }

    #[test]
    fn adjacent_snapped_rects_leave_no_gap() {
        let d = Display::new(1366, 768);
        let a = d.snap_rect(Rect::new(10.0, 0.0, 33.3, 10.0));
        let b = d.snap_rect(Rect::new(43.3, 0.0, 33.3, 10.0));
        assert!((a.right() - b.x).abs() < 1e-4);
    }
}
