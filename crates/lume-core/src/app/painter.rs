//! Drawing helpers shared by all screens: theme styles, text and icons.

use crate::display::Display;
use crate::render::{DisplayList, Rect};
use crate::text::{Align, TextEngine, TextStyle};
use crate::theme::{Color, ComponentTheme, ComputedStyle, Paint, StateFlags, THEME};
use crate::ui::style::mix;

/// The dimming of the wallpaper on the text screens: the theme's `--bg` at about 78% (the value
/// was tuned with a contrast measurement, see the tests). It is not a layer: the wallpaper is
/// drawn already scaled by the remaining 22% over the `--bg` the frame is cleared to (see
/// [`Painter::wallpaper`]), which costs nothing extra on a weak GPU.
pub const TEXT_SCREEN_DIM: Color = Color {
    r: 15,
    g: 23,
    b: 30,
    a: 200,
};

/// What a screen draws with for one frame.
pub struct Painter<'a> {
    pub list: &'a mut DisplayList,
    pub text: &'a mut TextEngine,
    pub display: Display,
}

impl Painter<'_> {
    /// Draws the wallpaper over the whole visible area, cropped to cover it (the image keeps its
    /// proportion). It goes first in every frame, behind everything else.
    ///
    /// `dimmed` darkens it towards `--bg` by [`TEXT_SCREEN_DIM`] without a second full-screen layer
    /// (that layer cost more than half of the frames on the TV): the image and the stars are
    /// drawn at the remaining share, and the frame is cleared to `--bg`, so the premultiplied
    /// blend gives `image * (1 - dim) + bg * dim`.
    pub fn wallpaper(&mut self, dimmed: bool) {
        self.wallpaper_at(crate::wallpaper::WallpaperId::default(), dimmed, 1.0);
    }

    /// The wallpaper at `intensity` (0.0, only the `--bg` the frame is cleared to, to 1.0, as
    /// [`wallpaper`](Self::wallpaper) draws it): the splash reveals it this way, by the color it is
    /// drawn with and not with a layer over it.
    pub fn wallpaper_at(
        &mut self,
        id: crate::wallpaper::WallpaperId,
        dimmed: bool,
        intensity: f32,
    ) {
        let keep = if dimmed {
            1.0 - f32::from(TEXT_SCREEN_DIM.a) / 255.0
        } else {
            1.0
        } * intensity.clamp(0.0, 1.0);
        if keep <= 0.0 {
            return;
        }
        let visible = self.display.visible_rect();
        let wallpaper = id.wallpaper();
        let uv = cover_uv(
            wallpaper.width as f32 / wallpaper.height as f32,
            visible.w / visible.h,
        );
        self.list
            .wallpaper_with_id(visible, uv, fade(Color::WHITE, keep), id);
        // The stars are drawn as points of 1 or 2 pixels of the render buffer, over the image, so
        // they stay as sharp as the interface on a 4K screen (which renders at 1080p).
        let pixel = 1.0 / self.display.scale;
        for &(sx, sy, brightness) in wallpaper.stars {
            let (u, v) = (f32::from(sx) / 1920.0, f32::from(sy) / 1080.0);
            if u < uv.x || u >= uv.right() || v < uv.y || v >= uv.bottom() {
                continue;
            }
            let size = if brightness > 204 { 2.0 * pixel } else { pixel };
            let x = self.display.snap(visible.x + (u - uv.x) / uv.w * visible.w);
            let y = self.display.snap(visible.y + (v - uv.y) / uv.h * visible.h);
            self.list.fill(
                Rect::new(x, y, size, size),
                0.0,
                Paint::Solid(Color {
                    r: 200,
                    g: 210,
                    b: 235,
                    a: (f32::from(brightness) * keep).round() as u8,
                }),
            );
        }
    }

    /// Re-samples the wallpaper through a rounded control with a small offset.
    /// This gives a cheap refractive cue without a full-screen offscreen buffer.
    pub fn wallpaper_glass(
        &mut self,
        rect: crate::render::Rect,
        radius: f32,
        id: crate::wallpaper::WallpaperId,
    ) {
        let visible = self.display.visible_rect();
        let wallpaper = id.wallpaper();
        let cover = cover_uv(
            wallpaper.width as f32 / wallpaper.height as f32,
            visible.w / visible.h,
        );
        let sample = crate::render::Rect::new(
            cover.x + (rect.x - visible.x + 4.0) / visible.w * cover.w,
            cover.y + (rect.y - visible.y - 3.0) / visible.h * cover.h,
            rect.w / visible.w * cover.w,
            rect.h / visible.h * cover.h,
        );
        self.list.glass_with_id(
            rect,
            sample,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 145,
            },
            radius,
            id,
        );
    }

    /// The theme style of `component` with the given pseudo-classes.
    pub fn style(&self, component: &ComponentTheme, state: StateFlags) -> ComputedStyle {
        THEME.resolve(component, state)
    }

    /// The style of `component` between its resting look (0.0) and its focused look (1.0).
    pub fn focus_style(&self, component: &ComponentTheme, progress: f32) -> ComputedStyle {
        let rest = THEME.resolve(component, StateFlags::NONE);
        if progress <= 0.0 {
            return rest;
        }
        mix(
            &rest,
            &THEME.resolve(component, StateFlags::FOCUS),
            progress,
        )
    }

    /// Draws a box (shadow, background and outline) as the style describes it.
    pub fn draw_box(&mut self, rect: Rect, style: &ComputedStyle) {
        self.list.draw_box(rect, style);
    }

    /// The text style a computed style asks for.
    pub fn text_style(style: &ComputedStyle) -> TextStyle {
        TextStyle {
            size: style.font_size,
            weight: style.font_weight,
            letter_spacing: style.letter_spacing,
        }
    }

    /// One line of text inside `rect`, ellipsized, aligned and centered vertically on its capitals.
    pub fn text(&mut self, text: &str, rect: Rect, align: Align, style: &ComputedStyle) {
        let text_style = Painter::text_style(style);
        let color = fade(style.color, style.opacity);
        self.text
            .draw_line(self.list, text, rect, align, &text_style, color);
    }

    /// Like [`Painter::text`] with an explicit color and style.
    pub fn text_with(
        &mut self,
        text: &str,
        rect: Rect,
        align: Align,
        style: &TextStyle,
        color: Color,
    ) {
        self.text
            .draw_line(self.list, text, rect, align, style, color);
    }

    /// The width `text` takes in the given style.
    pub fn measure(&mut self, text: &str, style: &ComputedStyle) -> f32 {
        self.text.measure(text, &Painter::text_style(style))
    }

    /// An icon from the icon font, centered in `rect` at `size`.
    pub fn icon(&mut self, icon: char, rect: Rect, size: f32, color: Color) {
        self.text.draw_icon(self.list, icon, rect, size, color);
    }
}

/// The part of an image (in 0..1 texture coordinates) that covers a rectangle of the given
/// proportions (width over height), centered: the image is cropped, never stretched or letterboxed.
pub fn cover_uv(image_aspect: f32, target_aspect: f32) -> Rect {
    if target_aspect > image_aspect {
        let h = image_aspect / target_aspect;
        Rect::new(0.0, (1.0 - h) / 2.0, 1.0, h)
    } else {
        let w = target_aspect / image_aspect;
        Rect::new((1.0 - w) / 2.0, 0.0, w, 1.0)
    }
}

pub fn fade(color: Color, opacity: f32) -> Color {
    Color {
        a: (f32::from(color.a) * opacity.clamp(0.0, 1.0)).round() as u8,
        ..color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_matching_proportion_uses_the_whole_image() {
        assert_eq!(
            cover_uv(16.0 / 9.0, 16.0 / 9.0),
            Rect::new(0.0, 0.0, 1.0, 1.0)
        );
    }

    #[test]
    fn a_wider_target_crops_top_and_bottom_and_a_taller_one_crops_the_sides() {
        let wide = cover_uv(16.0 / 9.0, 21.0 / 9.0);
        assert_eq!((wide.x, wide.w), (0.0, 1.0));
        assert!(
            (wide.h - (16.0 / 21.0)).abs() < 1e-5 && (wide.y * 2.0 + wide.h - 1.0).abs() < 1e-5
        );
        let tall = cover_uv(16.0 / 9.0, 4.0 / 3.0);
        assert_eq!((tall.y, tall.h), (0.0, 1.0));
        assert!((tall.w - (4.0 / 3.0) / (16.0 / 9.0)).abs() < 1e-5);
        assert!((tall.x * 2.0 + tall.w - 1.0).abs() < 1e-5);
    }
}
