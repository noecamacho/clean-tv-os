//! The display list: what to draw, in painter's order, in logical canvas units.

use super::geometry::Rect;
use crate::theme::{Color, ComputedStyle, Paint, Shadow};
use crate::wallpaper::WallpaperId;

/// How a rounded rectangle is painted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RectKind {
    /// The filled shape.
    Fill(Paint),
    /// Only a border of `width` inside the shape's edge.
    Ring { width: f32, color: Color },
    /// A soft shadow; `blur` is the CSS blur radius.
    Shadow { color: Color, blur: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectCmd {
    pub rect: Rect,
    pub radius: f32,
    pub kind: RectKind,
    /// Multiplies the alpha of everything the command paints.
    pub opacity: f32,
}

/// A textured quad: `uv` is in normalized atlas coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TexturedCmd {
    pub dest: Rect,
    pub uv: Rect,
    pub color: Color,
    pub radius: f32,
    /// What an image is composed over inside its shape: the item's own color, so the shape has
    /// one antialiased edge instead of one for a fill and another for the image over it.
    pub background: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cmd {
    Rect(RectCmd),
    /// A glyph from the text/icon atlas, tinted with `color`.
    Glyph(TexturedCmd),
    /// An RGBA image from the icon atlas, multiplied by `color` (white = unchanged).
    Image(TexturedCmd),
    /// A piece of the wallpaper texture (opaque RGB), multiplied by `color`.
    Wallpaper(TexturedCmd),
    /// A rounded, softly blurred and refracted sample of the wallpaper.
    Glass(TexturedCmd),
    /// A thumbnail from the wallpaper thumbnails atlas, with optional rounded corners.
    WallpaperThumb(TexturedCmd),
    PushClip(Rect),
    PopClip,
}

#[derive(Clone, Debug, Default)]
pub struct DisplayList {
    cmds: Vec<Cmd>,
    wallpaper: Option<WallpaperId>,
}

impl DisplayList {
    pub fn new() -> DisplayList {
        DisplayList::default()
    }

    pub fn cmds(&self) -> &[Cmd] {
        &self.cmds
    }

    pub fn wallpaper_id(&self) -> Option<WallpaperId> {
        self.wallpaper
    }

    pub fn clear(&mut self) {
        self.cmds.clear();
        self.wallpaper = None;
    }

    pub fn len(&self) -> usize {
        self.cmds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cmds.is_empty()
    }

    /// Appends a raw command.
    pub fn push(&mut self, cmd: Cmd) {
        self.cmds.push(cmd);
    }

    fn rect(&mut self, rect: Rect, radius: f32, kind: RectKind, opacity: f32) {
        self.cmds.push(Cmd::Rect(RectCmd {
            rect,
            radius,
            kind,
            opacity,
        }));
    }

    pub fn fill(&mut self, rect: Rect, radius: f32, paint: Paint) {
        self.rect(rect, radius, RectKind::Fill(paint), 1.0);
    }

    pub fn ring(&mut self, rect: Rect, radius: f32, width: f32, color: Color) {
        self.rect(rect, radius, RectKind::Ring { width, color }, 1.0);
    }

    pub fn shadow(&mut self, rect: Rect, radius: f32, color: Color, blur: f32) {
        self.rect(rect, radius, RectKind::Shadow { color, blur }, 1.0);
    }

    pub fn glyph(&mut self, dest: Rect, uv: Rect, color: Color) {
        self.cmds.push(Cmd::Glyph(TexturedCmd {
            dest,
            uv,
            color,
            radius: 0.0,
            background: Color::TRANSPARENT,
        }));
    }

    pub fn image(&mut self, dest: Rect, uv: Rect, color: Color) {
        self.image_rounded(dest, uv, color, 0.0);
    }

    pub fn image_rounded(&mut self, dest: Rect, uv: Rect, color: Color, radius: f32) {
        self.image_over(dest, uv, color, radius, Color::TRANSPARENT);
    }

    /// An image composed over `background` and clipped once by the rounded shape: `color` (a
    /// fade-in, for instance) only touches the image, and `background` shows wherever the image
    /// is transparent or not visible yet.
    pub fn image_over(
        &mut self,
        dest: Rect,
        uv: Rect,
        color: Color,
        radius: f32,
        background: Color,
    ) {
        self.cmds.push(Cmd::Image(TexturedCmd {
            dest,
            uv,
            color,
            radius,
            background,
        }));
    }

    pub fn wallpaper(&mut self, dest: Rect, uv: Rect, color: Color) {
        self.wallpaper_with_id(dest, uv, color, WallpaperId::Aurora);
    }

    pub fn wallpaper_with_id(&mut self, dest: Rect, uv: Rect, color: Color, id: WallpaperId) {
        self.wallpaper_rounded_with_id(dest, uv, color, 0.0, id);
    }

    pub fn wallpaper_rounded_with_id(
        &mut self,
        dest: Rect,
        uv: Rect,
        color: Color,
        radius: f32,
        id: WallpaperId,
    ) {
        self.wallpaper = Some(id);
        self.cmds.push(Cmd::Wallpaper(TexturedCmd {
            dest,
            uv,
            color,
            radius,
            background: Color::TRANSPARENT,
        }));
    }

    pub fn glass_with_id(
        &mut self,
        dest: Rect,
        uv: Rect,
        color: Color,
        radius: f32,
        id: WallpaperId,
    ) {
        self.wallpaper = Some(id);
        self.cmds.push(Cmd::Glass(TexturedCmd {
            dest,
            uv,
            color,
            radius,
            background: Color::TRANSPARENT,
        }));
    }

    pub fn wallpaper_thumb(&mut self, dest: Rect, uv: Rect, radius: f32) {
        self.cmds.push(Cmd::WallpaperThumb(TexturedCmd {
            dest,
            uv,
            color: Color::WHITE,
            radius,
            background: Color::TRANSPARENT,
        }));
    }

    /// Makes everything drawn since command `start` (an index from [`len`](Self::len)) `factor`
    /// (0.0 to 1.0) as opaque, for fading a part of the frame in or out without a layer over it.
    pub fn fade_from(&mut self, start: usize, factor: f32) {
        let factor = factor.clamp(0.0, 1.0);
        for cmd in self.cmds.iter_mut().skip(start) {
            match cmd {
                Cmd::Rect(r) => r.opacity *= factor,
                Cmd::Glyph(t)
                | Cmd::Image(t)
                | Cmd::Wallpaper(t)
                | Cmd::Glass(t)
                | Cmd::WallpaperThumb(t) => {
                    t.color.a = (f32::from(t.color.a) * factor).round() as u8;
                    t.background.a = (f32::from(t.background.a) * factor).round() as u8;
                }
                Cmd::PushClip(_) | Cmd::PopClip => {}
            }
        }
    }

    /// Moves commands `start..end` (indexes from [`len`](Self::len)) down by `dy`, clips included.
    pub fn translate_range(&mut self, start: usize, end: usize, dy: f32) {
        let end = end.min(self.cmds.len());
        for cmd in &mut self.cmds[start.min(end)..end] {
            match cmd {
                Cmd::Rect(r) => r.rect = r.rect.translate(0.0, dy),
                Cmd::Glyph(t)
                | Cmd::Image(t)
                | Cmd::Wallpaper(t)
                | Cmd::Glass(t)
                | Cmd::WallpaperThumb(t) => {
                    t.dest = t.dest.translate(0.0, dy);
                }
                Cmd::PushClip(rect) => *rect = rect.translate(0.0, dy),
                Cmd::PopClip => {}
            }
        }
    }

    /// Drops the commands from `start` on.
    pub fn truncate(&mut self, start: usize) {
        self.cmds.truncate(start);
    }

    pub fn push_clip(&mut self, rect: Rect) {
        self.cmds.push(Cmd::PushClip(rect));
    }

    pub fn pop_clip(&mut self) {
        self.cmds.push(Cmd::PopClip);
    }

    /// Draws a box the way its theme style describes it: shadow, fill, then outline.
    /// `rect` is the box before `style.scale`, which is applied here about its center.
    pub fn draw_box(&mut self, rect: Rect, style: &ComputedStyle) {
        self.draw_box_under(rect, style, 0.0);
    }

    /// Like [`draw_box`](Self::draw_box), for a box an image is drawn over afterwards: the outline
    /// also reaches `under` inside the box, where the image covers it, so the image's antialiased
    /// edge blends with the outline and not with what is behind (no dark line between them). From
    /// outside, the outline looks the same.
    pub fn draw_box_under(&mut self, rect: Rect, style: &ComputedStyle, under: f32) {
        let rect = rect.scaled_about_center(style.scale);
        let radius = style.border_radius * style.scale;
        let opacity = style.opacity.clamp(0.0, 1.0);
        if let Some(Shadow {
            offset_x,
            offset_y,
            blur,
            spread,
            color,
        }) = style.box_shadow
        {
            let shadow_rect = rect.translate(offset_x, offset_y).grow(spread);
            self.rect(
                shadow_rect,
                radius + spread.max(0.0),
                RectKind::Shadow { color, blur },
                opacity,
            );
        }
        if let Some(paint) = style.background {
            self.rect(rect, radius, RectKind::Fill(paint), opacity);
        }
        if let Some(outline) = style.outline {
            let grow = style.outline_offset + outline.width;
            self.rect(
                rect.grow(grow),
                radius + grow,
                RectKind::Ring {
                    width: outline.width + under,
                    color: outline.color,
                },
                opacity,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{StateFlags, THEME};

    #[test]
    fn focused_tile_draws_shadow_then_fill_then_ring() {
        let style = THEME.resolve(&THEME.tile, StateFlags::FOCUS);
        let mut list = DisplayList::new();
        let base = Rect::new(100.0, 100.0, 211.0, 119.0);
        list.draw_box(base, &style);

        let kinds: Vec<_> = list
            .cmds()
            .iter()
            .map(|c| match c {
                Cmd::Rect(r) => match r.kind {
                    RectKind::Shadow { .. } => "shadow",
                    RectKind::Fill(_) => "fill",
                    RectKind::Ring { .. } => "ring",
                },
                _ => "other",
            })
            .collect();
        // The tile has no background of its own in the theme: its color comes from the app.
        assert_eq!(kinds, ["shadow", "ring"]);

        // The ring surrounds the scaled box by outline-offset + width.
        let Cmd::Rect(ring) = list.cmds()[1] else {
            panic!()
        };
        let scaled = base.scaled_about_center(1.12);
        assert!((ring.rect.w - (scaled.w + 4.0)).abs() < 1e-3);
        assert!((ring.radius - (12.0 * 1.12 + 2.0)).abs() < 1e-3);
    }

    #[test]
    fn a_box_with_a_background_is_filled_between_shadow_and_ring() {
        let style = THEME.resolve(&THEME.editor_row, StateFlags::FOCUS);
        let mut list = DisplayList::new();
        list.draw_box(Rect::new(0.0, 0.0, 400.0, 71.0), &style);
        assert_eq!(list.len(), 2);
        assert!(matches!(
            list.cmds()[0],
            Cmd::Rect(RectCmd {
                kind: RectKind::Fill(_),
                ..
            })
        ));
        assert!(matches!(
            list.cmds()[1],
            Cmd::Rect(RectCmd {
                kind: RectKind::Ring { .. },
                ..
            })
        ));
    }
}
