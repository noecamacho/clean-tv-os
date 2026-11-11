//! Text and icon rendering: layout, measurement and an on-demand glyph atlas.
//!
//! Glyphs are rasterized on the CPU with `fontdue` at the final device size and only
//! when they are first drawn, so the atlas holds just what is on screen.
//! There is no shaping or right-to-left support yet: glyphs are laid out in logical
//! order with the font's kerning pairs.
//!
//! Per frame:
//! 1. [`TextEngine::begin_frame`]
//! 2. build the UI with [`TextEngine::draw`] and friends
//! 3. [`TextEngine::end_frame`]; if it returns `true` the atlas was rebuilt mid-frame,
//!    so the quads emitted so far point at stale texture coordinates and the UI must
//!    be built again (this settles after one extra pass)
//! 4. upload [`TextEngine::take_updates`] to the GPU

mod atlas;
mod fallback;
mod system_fonts;

pub use atlas::{Atlas, Slot, Update, MAX_HEIGHT, MAX_WIDTH};
pub use fallback::{FallbackFonts, NoFallback};
pub use system_fonts::{parse_fonts_xml, FontCandidate, SystemFonts};

use crate::fonts;
use crate::render::{DisplayList, Rect};
use crate::theme::Color;
use fontdue::{Font, FontSettings};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

const ELLIPSIS: char = '\u{2026}';

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextStyle {
    /// Font size in logical px.
    pub size: f32,
    /// CSS weight; the nearest embedded face is used.
    pub weight: u16,
    /// Extra space after every glyph, in logical px.
    pub letter_spacing: f32,
}

impl TextStyle {
    pub const fn new(size: f32, weight: u16) -> TextStyle {
        TextStyle {
            size,
            weight,
            letter_spacing: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
}

/// Vertical metrics of a style, in logical px.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMetrics {
    pub ascent: f32,
    pub descent: f32,
    /// Height of capital letters; text is centered vertically on it.
    pub cap_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum FontId {
    /// Index into the embedded text faces.
    Text(u8),
    Icons,
    /// Index into the fallback provider.
    Fallback(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct GlyphKey {
    font: FontId,
    glyph: u16,
    /// Device size in quarter pixels.
    size4: u32,
}

#[derive(Clone, Copy, Debug)]
struct GlyphEntry {
    /// `None` for glyphs without pixels, such as the space.
    slot: Option<Slot>,
    /// Offset of the bitmap's left edge from the pen, in device px.
    left: i32,
    /// Offset of the bitmap's bottom edge above the baseline, in device px.
    bottom: i32,
}

/// A glyph found for a character, before rasterization.
#[derive(Clone, Copy)]
struct Resolved {
    font: FontId,
    glyph: u16,
}

pub struct TextEngine {
    faces: Vec<(u16, Font)>,
    icons: Font,
    fallback: Box<dyn FallbackFonts>,
    atlas: Atlas,
    cache: HashMap<GlyphKey, GlyphEntry>,
    used: HashSet<GlyphKey>,
    scale: f32,
    /// Set when the atlas was rebuilt during the current frame.
    invalidated: bool,
}

impl TextEngine {
    pub fn new(scale: f32) -> TextEngine {
        TextEngine::with_fallback(scale, Box::new(NoFallback))
    }

    pub fn with_fallback(scale: f32, fallback: Box<dyn FallbackFonts>) -> TextEngine {
        let load = |data: &[u8]| {
            Font::from_bytes(data, FontSettings::default()).expect("embedded font is valid")
        };
        TextEngine {
            faces: fonts::TEXT_FACES
                .iter()
                .map(|(weight, data)| (*weight, load(data)))
                .collect(),
            icons: load(fonts::ICONS),
            fallback,
            atlas: Atlas::new(),
            cache: HashMap::new(),
            used: HashSet::new(),
            scale,
            invalidated: false,
        }
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    /// Changes the device scale. Every rasterized glyph is discarded.
    pub fn set_scale(&mut self, scale: f32) {
        if (scale - self.scale).abs() > f32::EPSILON {
            self.scale = scale;
            self.clear();
        }
    }

    /// Drops all rasterized glyphs (language or scale change) and starts a new atlas.
    pub fn clear(&mut self) {
        self.cache.clear();
        self.used.clear();
        self.atlas = Atlas::new();
        self.invalidated = true;
    }

    /// Releases memory held for sleeping: glyphs, the atlas bitmap and fallback fonts.
    pub fn release(&mut self) {
        self.clear();
        self.fallback.release();
    }

    pub fn atlas(&self) -> &Atlas {
        &self.atlas
    }

    /// Number of glyphs currently in the cache.
    pub fn cached_glyphs(&self) -> usize {
        self.cache.len()
    }

    pub fn begin_frame(&mut self) {
        self.used.clear();
        self.invalidated = false;
    }

    /// Returns `true` when the frame must be built again (see the module docs).
    pub fn end_frame(&mut self) -> bool {
        std::mem::take(&mut self.invalidated)
    }

    /// Pending GPU updates for the atlas.
    pub fn take_updates(&mut self) -> Vec<Update> {
        self.atlas.take_updates()
    }

    pub fn metrics(&self, style: &TextStyle) -> TextMetrics {
        let px = style.size * self.scale;
        let font = &self.faces[self.face_index(style.weight)].1;
        let line = font
            .horizontal_line_metrics(px)
            .expect("the font has horizontal metrics");
        let cap = font.metrics('H', px).height as f32;
        TextMetrics {
            ascent: line.ascent / self.scale,
            descent: -line.descent / self.scale,
            cap_height: cap / self.scale,
        }
    }

    /// Width of `text` in logical px, without drawing it.
    pub fn measure(&mut self, text: &str, style: &TextStyle) -> f32 {
        let (width, _) = self.pen_positions(text, style, |_, _, _| {});
        width / self.scale
    }

    /// Shortens `text` with an ellipsis until it fits in `max_width` logical px.
    pub fn ellipsize<'a>(
        &mut self,
        text: &'a str,
        style: &TextStyle,
        max_width: f32,
    ) -> Cow<'a, str> {
        if text.is_empty() || max_width <= 0.0 {
            return Cow::Borrowed("");
        }

        let px = style.size * self.scale;
        let spacing = style.letter_spacing * self.scale;
        let ellipsis_res = self.resolve(ELLIPSIS, style.weight);
        let ellipsis_font = self.font(ellipsis_res.font);
        let ellipsis_advance = ellipsis_font
            .metrics_indexed(ellipsis_res.glyph, px)
            .advance_width;
        let ellipsis_w = (ellipsis_advance + spacing) / self.scale;

        let mut pen = 0.0f32;
        let mut previous: Option<Resolved> = None;
        let mut best_end = 0usize;

        let mut pen_at_trimmed = 0.0f32;
        let mut last_non_ws_res: Option<Resolved> = None;
        let mut last_non_ws_end = 0usize;

        for (idx, ch) in text.char_indices() {
            let resolved = self.resolve(ch, style.weight);
            let font = self.font(resolved.font);
            if let Some(prev) = previous.filter(|p| p.font == resolved.font) {
                if let Some(kern) = font.horizontal_kern_indexed(prev.glyph, resolved.glyph, px) {
                    pen += kern;
                }
            }
            let advance = font.metrics_indexed(resolved.glyph, px).advance_width;
            pen += advance + spacing;
            previous = Some(resolved);

            let char_end = idx + ch.len_utf8();
            if !ch.is_whitespace() {
                last_non_ws_end = char_end;
                pen_at_trimmed = pen;
                last_non_ws_res = Some(resolved);
            }

            if last_non_ws_end > 0 {
                let mut cand_pen = pen_at_trimmed;
                if let Some(prev) = last_non_ws_res.filter(|p| p.font == ellipsis_res.font) {
                    let e_font = self.font(ellipsis_res.font);
                    if let Some(kern) =
                        e_font.horizontal_kern_indexed(prev.glyph, ellipsis_res.glyph, px)
                    {
                        cand_pen += kern;
                    }
                }
                cand_pen += ellipsis_advance + spacing;
                let cand_w = cand_pen / self.scale;
                if cand_w <= max_width {
                    best_end = last_non_ws_end;
                }
            }
        }

        let width = pen / self.scale;
        if width <= max_width {
            Cow::Borrowed(text)
        } else if ellipsis_w > max_width {
            Cow::Borrowed("")
        } else if best_end == 0 {
            Cow::Borrowed("…")
        } else {
            Cow::Owned(format!("{}…", &text[..best_end]))
        }
    }

    fn pen_positions_and_width(
        &mut self,
        text: &str,
        style: &TextStyle,
    ) -> (Vec<(GlyphKey, f32)>, f32) {
        let mut quads = Vec::new();
        let (pen, _) = self.pen_positions(text, style, |key, pen, _| quads.push((key, pen)));
        let width = pen / self.scale;
        (quads, width)
    }

    fn emit_quads(
        &mut self,
        list: &mut DisplayList,
        quads: Vec<(GlyphKey, f32)>,
        x: f32,
        baseline_y: f32,
        color: Color,
    ) {
        let scale = self.scale;
        let origin = x * scale;
        let baseline = (baseline_y * scale).round();
        for (key, pen) in quads {
            let Some(entry) = self.glyph(key) else {
                continue;
            };
            let Some(slot) = entry.slot else { continue };
            let left = (origin + pen).round() + entry.left as f32;
            let top = baseline - entry.bottom as f32 - slot.height as f32;
            let (aw, ah) = (self.atlas.width() as f32, self.atlas.height() as f32);
            list.glyph(
                Rect::new(
                    left / scale,
                    top / scale,
                    slot.width as f32 / scale,
                    slot.height as f32 / scale,
                ),
                Rect::new(
                    slot.x as f32 / aw,
                    slot.y as f32 / ah,
                    slot.width as f32 / aw,
                    slot.height as f32 / ah,
                ),
                color,
            );
        }
    }

    /// Draws `text` with its pen starting at `x` on the baseline `baseline_y` (logical px).
    pub fn draw(
        &mut self,
        list: &mut DisplayList,
        text: &str,
        x: f32,
        baseline_y: f32,
        style: &TextStyle,
        color: Color,
    ) {
        let (quads, _) = self.pen_positions_and_width(text, style);
        self.emit_quads(list, quads, x, baseline_y, color);
    }

    /// Draws one line inside `rect`, ellipsized to its width, aligned horizontally
    /// and centered vertically on the capital letters.
    pub fn draw_line(
        &mut self,
        list: &mut DisplayList,
        text: &str,
        rect: Rect,
        align: Align,
        style: &TextStyle,
        color: Color,
    ) {
        let text = self.ellipsize(text, style, rect.w);
        let (quads, width) = self.pen_positions_and_width(&text, style);
        let x = match align {
            Align::Start => rect.x,
            Align::Center => rect.x + (rect.w - width) / 2.0,
            Align::End => rect.right() - width,
        };
        let metrics = self.metrics(style);
        let baseline = rect.center().1 + metrics.cap_height / 2.0;
        self.emit_quads(list, quads, x, baseline, color);
    }

    /// Draws an icon from the icon font with its bitmap centered in `rect`. `size` is the font
    /// size in logical px (Lucide icons are drawn in a square of that size).
    pub fn draw_icon(
        &mut self,
        list: &mut DisplayList,
        icon: char,
        rect: Rect,
        size: f32,
        color: Color,
    ) {
        let scale = self.scale;
        let glyph = self.icons.lookup_glyph_index(icon);
        if glyph == 0 {
            return;
        }
        let key = GlyphKey {
            font: FontId::Icons,
            glyph,
            size4: (size * scale * 4.0).round() as u32,
        };
        let Some(entry) = self.glyph(key) else { return };
        let Some(slot) = entry.slot else { return };
        let (center_x, center_y) = rect.center();
        let left = (center_x * scale - slot.width as f32 / 2.0).round();
        let top = (center_y * scale - slot.height as f32 / 2.0).round();
        let (aw, ah) = (self.atlas.width() as f32, self.atlas.height() as f32);
        list.glyph(
            Rect::new(
                left / scale,
                top / scale,
                slot.width as f32 / scale,
                slot.height as f32 / scale,
            ),
            Rect::new(
                slot.x as f32 / aw,
                slot.y as f32 / ah,
                slot.width as f32 / aw,
                slot.height as f32 / ah,
            ),
            color,
        );
    }

    fn face_index(&self, weight: u16) -> usize {
        // Nearest weight; the heavier face wins a tie.
        self.faces
            .iter()
            .enumerate()
            .rev()
            .min_by_key(|(_, (w, _))| w.abs_diff(weight))
            .map(|(i, _)| i)
            .expect("there are text faces")
    }

    fn font(&self, id: FontId) -> &Font {
        match id {
            FontId::Text(i) => &self.faces[usize::from(i)].1,
            FontId::Icons => &self.icons,
            FontId::Fallback(i) => self.fallback.font(usize::from(i)),
        }
    }

    fn resolve(&mut self, c: char, weight: u16) -> Resolved {
        if ('\u{e000}'..='\u{f8ff}').contains(&c) {
            let glyph = self.icons.lookup_glyph_index(c);
            if glyph != 0 {
                return Resolved {
                    font: FontId::Icons,
                    glyph,
                };
            }
        }
        let face = self.face_index(weight);
        let glyph = self.faces[face].1.lookup_glyph_index(c);
        if glyph != 0 {
            return Resolved {
                font: FontId::Text(face as u8),
                glyph,
            };
        }
        if let Some(index) = self.fallback.find(c) {
            let glyph = self.fallback.font(index).lookup_glyph_index(c);
            if glyph != 0 {
                return Resolved {
                    font: FontId::Fallback(index as u16),
                    glyph,
                };
            }
        }
        // The font's own "missing glyph" shape.
        Resolved {
            font: FontId::Text(face as u8),
            glyph: 0,
        }
    }

    /// Walks the text calling `emit(key, pen_x, advance)` per glyph (device px) and
    /// returns the total width and the number of glyphs.
    fn pen_positions(
        &mut self,
        text: &str,
        style: &TextStyle,
        mut emit: impl FnMut(GlyphKey, f32, f32),
    ) -> (f32, usize) {
        let px = style.size * self.scale;
        let size4 = (px * 4.0).round() as u32;
        let spacing = style.letter_spacing * self.scale;
        let mut pen = 0.0f32;
        let mut previous: Option<Resolved> = None;
        let mut count = 0;
        for c in text.chars() {
            let resolved = self.resolve(c, style.weight);
            let font = self.font(resolved.font);
            if let Some(prev) = previous.filter(|p| p.font == resolved.font) {
                if let Some(kern) = font.horizontal_kern_indexed(prev.glyph, resolved.glyph, px) {
                    pen += kern;
                }
            }
            let advance = font.metrics_indexed(resolved.glyph, px).advance_width;
            emit(
                GlyphKey {
                    font: resolved.font,
                    glyph: resolved.glyph,
                    size4,
                },
                pen,
                advance,
            );
            pen += advance + spacing;
            previous = Some(resolved);
            count += 1;
        }
        (pen, count)
    }

    /// Looks up or rasterizes a glyph, rebuilding the atlas when it is full.
    fn glyph(&mut self, key: GlyphKey) -> Option<GlyphEntry> {
        self.used.insert(key);
        if let Some(entry) = self.cache.get(&key) {
            return Some(*entry);
        }
        if let Some(entry) = self.rasterize(key) {
            self.cache.insert(key, entry);
            return Some(entry);
        }
        // The atlas is full at its maximum size: start over with the glyphs of this frame.
        self.rebuild();
        let entry = self.rasterize(key)?;
        self.cache.insert(key, entry);
        Some(entry)
    }

    fn rasterize(&mut self, key: GlyphKey) -> Option<GlyphEntry> {
        let px = key.size4 as f32 / 4.0;
        let (metrics, mut bitmap) = self.font(key.font).rasterize_indexed(key.glyph, px);
        let (mut width, mut height) = (metrics.width as u32, metrics.height as u32);
        let (mut left, mut bottom) = (metrics.xmin, metrics.ymin);
        if key.font == FontId::Icons && width > 0 && height > 0 {
            // The icons' edges get the theme's softness, like every shape (the text stays as is).
            bitmap = soften(&bitmap, width, height, crate::theme::vars::EDGE_SOFTNESS);
            (width, height, left, bottom) = (width + 2, height + 2, left - 1, bottom - 1);
        }
        let slot = if width == 0 || height == 0 {
            None
        } else {
            Some(self.atlas.insert(width, height, &bitmap)?)
        };
        Some(GlyphEntry { slot, left, bottom })
    }

    fn rebuild(&mut self) {
        self.atlas = Atlas::new();
        self.cache.clear();
        self.invalidated = true;
        let keys: Vec<GlyphKey> = self.used.iter().copied().collect();
        for key in keys {
            if let Some(entry) = self.rasterize(key) {
                self.cache.insert(key, entry);
            }
        }
    }
}

/// Widens the antialiased edge of a coverage bitmap to `softness` pixels, the ramp the shaders give
/// every shape: a 3-tap filter `[a, 1 - 2a, a]` across and down, whose spread added to the one pixel
/// of the rasterizer's own ramp makes `softness` (variances `1/12 + 2a = softness² / 12`). The
/// bitmap grows by one pixel on every side for the spread.
fn soften(bitmap: &[u8], width: u32, height: u32, softness: f32) -> Vec<u8> {
    let a = ((softness * softness - 1.0) / 24.0).clamp(0.0, 1.0 / 3.0);
    let (w, h) = (width as usize + 2, height as usize + 2);
    let mut padded = vec![0.0f32; w * h];
    for y in 0..height as usize {
        for x in 0..width as usize {
            padded[(y + 1) * w + x + 1] = f32::from(bitmap[y * width as usize + x]);
        }
    }
    let at = |v: &[f32], x: usize, y: usize, dx: isize, dy: isize| {
        let (x, y) = (x as isize + dx, y as isize + dy);
        if x < 0 || y < 0 || x >= w as isize || y >= h as isize {
            0.0
        } else {
            v[y as usize * w + x as usize]
        }
    };
    let mut across = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            across[y * w + x] = a * at(&padded, x, y, -1, 0)
                + (1.0 - 2.0 * a) * at(&padded, x, y, 0, 0)
                + a * at(&padded, x, y, 1, 0);
        }
    }
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let v = a * at(&across, x, y, 0, -1)
                + (1.0 - 2.0 * a) * at(&across, x, y, 0, 0)
                + a * at(&across, x, y, 0, 1);
            out.push(v.round().clamp(0.0, 255.0) as u8);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::Cmd;
    use crate::theme::Color;

    const BODY: TextStyle = TextStyle::new(20.0, 500);

    fn glyph_quads(list: &DisplayList) -> Vec<(Rect, Rect)> {
        list.cmds()
            .iter()
            .filter_map(|c| match c {
                Cmd::Glyph(g) => Some((g.dest, g.uv)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn measure_grows_with_text_and_size() {
        let mut engine = TextEngine::new(1.0);
        let short = engine.measure("lume", &BODY);
        let long = engine.measure("lume launcher", &BODY);
        let big = engine.measure("lume", &TextStyle::new(40.0, 500));
        assert!(short > 20.0 && long > short);
        assert!(
            (big / short - 2.0).abs() < 0.05,
            "doubling the size doubles the width ({big} vs {short})"
        );
        assert_eq!(engine.measure("", &BODY), 0.0);
    }

    #[test]
    fn measure_does_not_touch_the_atlas() {
        let mut engine = TextEngine::new(1.0);
        engine.measure("Documentários para maratonar", &BODY);
        assert_eq!(engine.cached_glyphs(), 0);
    }

    #[test]
    fn kerning_tightens_classic_pairs() {
        let mut engine = TextEngine::new(2.0);
        let style = TextStyle::new(40.0, 700);
        let pair = engine.measure("AV", &style);
        let separate = engine.measure("A", &style) + engine.measure("V", &style);
        assert!(pair < separate - 0.5, "AV measures {pair}, A+V {separate}");
    }

    #[test]
    fn letter_spacing_adds_per_glyph() {
        let mut engine = TextEngine::new(1.0);
        let base = engine.measure("abcd", &BODY);
        let spaced = engine.measure(
            "abcd",
            &TextStyle {
                letter_spacing: 2.0,
                ..BODY
            },
        );
        assert!((spaced - base - 8.0).abs() < 1e-3);
    }

    #[test]
    fn heavier_weights_are_wider() {
        let mut engine = TextEngine::new(1.0);
        let medium = engine.measure("Netflix", &TextStyle::new(22.0, 500));
        let extra = engine.measure("Netflix", &TextStyle::new(22.0, 800));
        assert!(extra > medium);
    }

    #[test]
    fn drawing_adds_glyphs_on_demand_and_reuses_them() {
        let mut engine = TextEngine::new(1.0);
        let mut list = DisplayList::new();
        engine.begin_frame();
        engine.draw(&mut list, "Hello", 10.0, 100.0, &BODY, Color::WHITE);
        // H, e, l, o: "l" repeats.
        assert_eq!(engine.cached_glyphs(), 4);
        assert_eq!(glyph_quads(&list).len(), 5);
        engine.draw(&mut list, "Hello", 10.0, 140.0, &BODY, Color::WHITE);
        assert_eq!(engine.cached_glyphs(), 4, "the second draw uses the cache");
        assert!(!engine.end_frame());
    }

    #[test]
    fn spaces_have_no_quad_but_advance_the_pen() {
        let mut engine = TextEngine::new(1.0);
        let mut list = DisplayList::new();
        engine.draw(&mut list, "a b", 0.0, 50.0, &BODY, Color::WHITE);
        let quads = glyph_quads(&list);
        assert_eq!(quads.len(), 2);
        assert!(quads[1].0.x - quads[0].0.right() > 3.0);
    }

    #[test]
    fn glyphs_sit_on_the_baseline() {
        let mut engine = TextEngine::new(1.0);
        let mut list = DisplayList::new();
        engine.draw(
            &mut list,
            "H",
            0.0,
            100.0,
            &TextStyle::new(40.0, 700),
            Color::WHITE,
        );
        let (dest, _) = glyph_quads(&list)[0];
        // Capital H stands on the baseline and is about as tall as the cap height.
        assert!(
            (dest.bottom() - 100.0).abs() < 1.0,
            "bottom is {}",
            dest.bottom()
        );
        let cap = engine.metrics(&TextStyle::new(40.0, 700)).cap_height;
        assert!(
            (dest.h - cap).abs() < 1.5 && (cap - 0.745 * 40.0).abs() < 1.5,
            "h={} cap={cap}",
            dest.h
        );
    }

    #[test]
    fn quads_are_pixel_aligned_at_any_scale() {
        for scale in [1.0, 1.5, 0.75] {
            let mut engine = TextEngine::new(scale);
            let mut list = DisplayList::new();
            engine.draw(&mut list, "Prime Video", 13.3, 77.7, &BODY, Color::WHITE);
            for (dest, _) in glyph_quads(&list) {
                for v in [
                    dest.x * scale,
                    dest.y * scale,
                    dest.w * scale,
                    dest.h * scale,
                ] {
                    assert!(
                        (v - v.round()).abs() < 1e-3,
                        "scale {scale}: {v} is not a whole pixel"
                    );
                }
            }
        }
    }

    #[test]
    fn uv_rectangles_match_the_atlas_bitmap() {
        let mut engine = TextEngine::new(1.0);
        let mut list = DisplayList::new();
        engine.draw(
            &mut list,
            "W",
            0.0,
            50.0,
            &TextStyle::new(30.0, 700),
            Color::WHITE,
        );
        let (dest, uv) = glyph_quads(&list)[0];
        let atlas = engine.atlas();
        assert!((uv.w * atlas.width() as f32 - dest.w).abs() < 1e-3);
        assert!((uv.h * atlas.height() as f32 - dest.h).abs() < 1e-3);
        // The covered texels hold glyph pixels.
        let slot = Slot {
            x: (uv.x * atlas.width() as f32).round() as u32,
            y: (uv.y * atlas.height() as f32).round() as u32,
            width: dest.w as u32,
            height: dest.h as u32,
        };
        assert!(atlas.region_pixels(slot).iter().any(|&p| p > 200));
    }

    #[test]
    fn ellipsize_keeps_text_that_fits() {
        let mut engine = TextEngine::new(1.0);
        let width = engine.measure("Netflix", &BODY);
        assert_eq!(engine.ellipsize("Netflix", &BODY, width + 1.0), "Netflix");
    }

    #[test]
    fn ellipsize_truncates_with_an_ellipsis_that_fits() {
        let mut engine = TextEngine::new(1.0);
        let text = "The Mandalorian: Season Three Behind The Scenes";
        for max in [60.0, 120.0, 200.0, 300.0] {
            let out = engine.ellipsize(text, &BODY, max).into_owned();
            assert!(out.ends_with('…'), "{out:?}");
            assert!(out.len() < text.len());
            assert!(
                engine.measure(&out, &BODY) <= max,
                "{out:?} is wider than {max}"
            );
        }
        // It keeps as much as possible: one more character would not fit.
        let out = engine.ellipsize(text, &BODY, 200.0).into_owned();
        let kept = out.trim_end_matches('…').chars().count();
        let longer: String = text.chars().take(kept + 1).collect::<String>() + "…";
        assert!(engine.measure(&longer, &BODY) > 200.0);
    }

    #[test]
    fn ellipsize_gives_up_when_even_the_ellipsis_does_not_fit() {
        let mut engine = TextEngine::new(1.0);
        assert_eq!(engine.ellipsize("Netflix", &BODY, 3.0), "");
    }

    #[test]
    fn ellipsize_handles_multibyte_text() {
        let mut engine = TextEngine::new(1.0);
        let out = engine
            .ellipsize("Configurações avançadas do sistema", &BODY, 150.0)
            .into_owned();
        assert!(out.ends_with('…') && engine.measure(&out, &BODY) <= 150.0);
    }

    #[test]
    fn ellipsize_single_pass_matches_exact_boundaries() {
        let mut engine = TextEngine::new(1.0);
        let text = "Hello world";
        let full_width = engine.measure(text, &BODY);

        // Fits exactly -> Cow::Borrowed
        let res = engine.ellipsize(text, &BODY, full_width);
        assert!(matches!(res, Cow::Borrowed(_)));
        assert_eq!(res, "Hello world");

        // Long string truncated with ellipsis -> engine.measure <= max_width
        let long = "Um Aplicativo Muito Longo Para Caber No Retângulo";
        let res_long = engine.ellipsize(long, &BODY, 120.0);
        assert!(engine.measure(&res_long, &BODY) <= 120.0);
        assert!(res_long.ends_with('…'));

        // Trailing whitespace before ellipsis is trimmed ("Hello world" -> "Hello…")
        let hello_w = engine.measure("Hello", &BODY);
        let ellipsis_w = engine.measure("…", &BODY);
        let res_trimmed = engine.ellipsize("Hello world", &BODY, hello_w + ellipsis_w + 1.0);
        assert_eq!(res_trimmed, "Hello…");
        assert_ne!(res_trimmed, "Hello …");
    }

    #[test]
    fn draw_line_aligns_inside_the_rect() {
        let mut engine = TextEngine::new(1.0);
        let rect = Rect::new(100.0, 100.0, 300.0, 40.0);
        let width = engine.measure("Apps", &BODY);
        let mut first_x = |align| {
            let mut list = DisplayList::new();
            engine.draw_line(&mut list, "Apps", rect, align, &BODY, Color::WHITE);
            glyph_quads(&list)[0].0.x
        };
        let (start, center, end) = (
            first_x(Align::Start),
            first_x(Align::Center),
            first_x(Align::End),
        );
        assert!((start - 100.0).abs() < 3.0);
        assert!((center - (100.0 + (300.0 - width) / 2.0)).abs() < 3.0);
        assert!((end - (400.0 - width)).abs() < 4.0);
    }

    #[test]
    fn draw_line_centers_capitals_vertically() {
        let mut engine = TextEngine::new(1.0);
        let rect = Rect::new(0.0, 100.0, 300.0, 60.0);
        let mut list = DisplayList::new();
        engine.draw_line(
            &mut list,
            "H",
            rect,
            Align::Start,
            &TextStyle::new(30.0, 700),
            Color::WHITE,
        );
        let (dest, _) = glyph_quads(&list)[0];
        assert!(
            (dest.center().1 - 130.0).abs() < 1.5,
            "center is {}",
            dest.center().1
        );
    }

    #[test]
    fn icons_are_drawn_from_the_icon_font() {
        let mut engine = TextEngine::new(1.0);
        let mut list = DisplayList::new();
        engine.draw(
            &mut list,
            &crate::icons::SETTINGS.to_string(),
            0.0,
            40.0,
            &TextStyle::new(24.0, 500),
            Color::WHITE,
        );
        let quads = glyph_quads(&list);
        assert_eq!(quads.len(), 1);
        assert!(quads[0].0.w > 10.0, "the gear icon has a visible size");
    }

    #[test]
    fn soften_spreads_the_edge_by_the_theme_softness_and_keeps_the_ink() {
        // A hard 4x4 square of full coverage in a 6x6 bitmap.
        let mut square = vec![0u8; 36];
        for y in 1..5 {
            for x in 1..5 {
                square[y * 6 + x] = 255;
            }
        }
        let soft = soften(&square, 6, 6, 1.5);
        assert_eq!(soft.len(), 8 * 8, "one more pixel on every side");
        // The ink is the same (within rounding), spread over a wider edge.
        let ink = |b: &[u8]| b.iter().map(|&v| u32::from(v)).sum::<u32>();
        assert!(ink(&soft).abs_diff(ink(&square)) <= 64);
        let partial = |b: &[u8]| b.iter().filter(|&&v| v > 0 && v < 255).count();
        assert!(partial(&soft) > partial(&square));
        // The middle stays fully covered and the new border stays empty.
        assert_eq!(soft[4 * 8 + 4], 255);
        assert!(soft[..8].iter().all(|&v| v == 0));
        // With a ramp of 1 pixel, the rasterizer's own, nothing changes.
        assert_eq!(&soften(&square, 6, 6, 1.0)[9..15], &[0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn icons_are_softened_and_text_is_not() {
        let mut engine = TextEngine::new(1.5);
        let style = TextStyle::new(22.0, 500);
        let icon = engine
            .glyph(GlyphKey {
                font: FontId::Icons,
                glyph: engine.icons.lookup_glyph_index(crate::icons::SETTINGS),
                size4: (22.0 * 1.5 * 4.0) as u32,
            })
            .and_then(|e| e.slot)
            .unwrap();
        let (metrics, _) = engine.icons.rasterize_indexed(
            engine.icons.lookup_glyph_index(crate::icons::SETTINGS),
            33.0,
        );
        assert_eq!(icon.width, metrics.width as u32 + 2);
        let mut list = DisplayList::new();
        engine.draw(&mut list, "H", 0.0, 40.0, &style, Color::WHITE);
        let face = engine.face_index(500);
        let h = engine.faces[face].1.lookup_glyph_index('H');
        let (text_metrics, _) = engine.faces[face].1.rasterize_indexed(h, 33.0);
        let quad = glyph_quads(&list)[0].0;
        assert_eq!((quad.w * 1.5).round() as usize, text_metrics.width);
    }

    #[test]
    fn icons_are_centered_in_their_rect() {
        let mut engine = TextEngine::new(1.0);
        for icon in [
            crate::icons::SETTINGS,
            crate::icons::CHEVRON_UP,
            crate::icons::TRASH,
            crate::icons::PLUS,
        ] {
            let mut list = DisplayList::new();
            let rect = Rect::new(100.0, 200.0, 48.0, 48.0);
            engine.draw_icon(&mut list, icon, rect, 22.0, Color::WHITE);
            let quads = glyph_quads(&list);
            assert_eq!(quads.len(), 1, "{icon:?}");
            let (dest, _) = quads[0];
            assert!(
                (dest.center().0 - 124.0).abs() <= 0.6 && (dest.center().1 - 224.0).abs() <= 0.6,
                "{icon:?} at {dest:?}"
            );
        }
    }

    #[test]
    fn missing_characters_fall_back_to_the_font_missing_glyph() {
        let mut engine = TextEngine::new(1.0);
        let mut list = DisplayList::new();
        // Cyrillic and CJK are not in the embedded subset.
        engine.draw(&mut list, "Яблоко 日本", 0.0, 40.0, &BODY, Color::WHITE);
        assert!(
            !glyph_quads(&list).is_empty(),
            "something is drawn, and nothing panics"
        );
        assert!(engine.measure("Яблоко", &BODY) > 20.0);
    }

    #[test]
    fn changing_the_scale_discards_the_cache() {
        let mut engine = TextEngine::new(1.0);
        let mut list = DisplayList::new();
        engine.draw(&mut list, "lume", 0.0, 40.0, &BODY, Color::WHITE);
        assert!(engine.cached_glyphs() > 0);
        engine.set_scale(1.5);
        assert_eq!(engine.cached_glyphs(), 0);
        assert_eq!(
            engine.take_updates(),
            [Update::Full {
                width: 512,
                height: 256
            }]
        );
    }

    #[test]
    fn a_full_atlas_is_rebuilt_with_the_visible_glyphs() {
        let mut engine = TextEngine::new(1.0);
        // Fill the atlas with ever larger glyphs, one size per frame.
        let sample = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let mut list = DisplayList::new();
        let mut size = 30.0;
        let mut cached_before = 0;
        loop {
            engine.begin_frame();
            list.clear();
            cached_before = engine.cached_glyphs().max(cached_before);
            engine.draw(
                &mut list,
                sample,
                0.0,
                100.0,
                &TextStyle::new(size, 700),
                Color::WHITE,
            );
            if engine.end_frame() {
                break;
            }
            size += 1.0;
            assert!(size < 400.0, "the atlas never filled up");
        }
        // The rebuild kept only the glyphs of the frame that overflowed.
        assert!(
            engine.cached_glyphs() <= 52,
            "{} glyphs kept",
            engine.cached_glyphs()
        );
        assert!(engine.cached_glyphs() < cached_before);
        assert!(matches!(
            engine.take_updates().first(),
            Some(Update::Full { .. })
        ));

        // The next frame, built again as end_frame asked, draws everything.
        engine.begin_frame();
        list.clear();
        engine.draw(
            &mut list,
            sample,
            0.0,
            100.0,
            &TextStyle::new(size, 700),
            Color::WHITE,
        );
        assert!(!engine.end_frame(), "the second pass fits");
        assert_eq!(glyph_quads(&list).len(), 52);
        assert!(engine.atlas().width() <= MAX_WIDTH && engine.atlas().height() <= MAX_HEIGHT);
    }

    #[test]
    fn release_frees_the_atlas() {
        let mut engine = TextEngine::new(1.0);
        let mut list = DisplayList::new();
        engine.draw(&mut list, "lume", 0.0, 40.0, &BODY, Color::WHITE);
        engine.release();
        assert_eq!(engine.cached_glyphs(), 0);
    }
}
