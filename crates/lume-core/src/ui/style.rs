//! Blending between two computed styles, for animated focus.

use crate::theme::{Color, ComputedStyle, Outline, Paint, Shadow};

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    lerp(f32::from(a), f32::from(b), t)
        .round()
        .clamp(0.0, 255.0) as u8
}

pub fn mix_color(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: lerp_u8(a.r, b.r, t),
        g: lerp_u8(a.g, b.g, t),
        b: lerp_u8(a.b, b.b, t),
        a: lerp_u8(a.a, b.a, t),
    }
}

fn fade(color: Color, t: f32) -> Color {
    Color {
        a: (f32::from(color.a) * t.clamp(0.0, 1.0)).round() as u8,
        ..color
    }
}

fn mix_paint(a: Option<Paint>, b: Option<Paint>, t: f32) -> Option<Paint> {
    match (a, b) {
        (Some(Paint::Solid(x)), Some(Paint::Solid(y))) => Some(Paint::Solid(mix_color(x, y, t))),
        // Gradients (or a background that appears or disappears) switch at the halfway point.
        (a, b) => {
            if t < 0.5 {
                a
            } else {
                b
            }
        }
    }
}

fn mix_outline(a: Option<Outline>, b: Option<Outline>, t: f32) -> Option<Outline> {
    match (a, b) {
        (None, None) => None,
        (Some(x), Some(y)) => Some(Outline {
            width: lerp(x.width, y.width, t),
            color: mix_color(x.color, y.color, t),
        }),
        // An outline that appears or disappears fades in or out at its own width.
        (None, Some(y)) => Some(Outline {
            color: fade(y.color, t),
            ..y
        }),
        (Some(x), None) => Some(Outline {
            color: fade(x.color, 1.0 - t),
            ..x
        }),
    }
}

fn mix_shadow(a: Option<Shadow>, b: Option<Shadow>, t: f32) -> Option<Shadow> {
    match (a, b) {
        (None, None) => None,
        (Some(x), Some(y)) => Some(Shadow {
            offset_x: lerp(x.offset_x, y.offset_x, t),
            offset_y: lerp(x.offset_y, y.offset_y, t),
            blur: lerp(x.blur, y.blur, t),
            spread: lerp(x.spread, y.spread, t),
            color: mix_color(x.color, y.color, t),
        }),
        (None, Some(y)) => Some(Shadow {
            color: fade(y.color, t),
            ..y
        }),
        (Some(x), None) => Some(Shadow {
            color: fade(x.color, 1.0 - t),
            ..x
        }),
    }
}

/// The style `t` of the way from `a` (0.0) to `b` (1.0): numbers and colors are interpolated,
/// outlines and shadows fade in or out, and non-numeric properties come from `b` once past half.
pub fn mix(a: &ComputedStyle, b: &ComputedStyle, t: f32) -> ComputedStyle {
    let t = t.clamp(0.0, 1.0);
    let late = if t < 0.5 { a } else { b };
    ComputedStyle {
        background: mix_paint(a.background, b.background, t),
        color: mix_color(a.color, b.color, t),
        opacity: lerp(a.opacity, b.opacity, t),
        border_radius: lerp(a.border_radius, b.border_radius, t),
        outline: mix_outline(a.outline, b.outline, t),
        outline_offset: lerp(a.outline_offset, b.outline_offset, t),
        box_shadow: mix_shadow(a.box_shadow, b.box_shadow, t),
        font_size: lerp(a.font_size, b.font_size, t),
        font_weight: late.font_weight,
        letter_spacing: lerp(a.letter_spacing, b.letter_spacing, t),
        padding: late.padding,
        gap: late.gap,
        width: match (a.width, b.width) {
            (Some(x), Some(y)) => Some(lerp(x, y, t)),
            _ => late.width,
        },
        height: match (a.height, b.height) {
            (Some(x), Some(y)) => Some(lerp(x, y, t)),
            _ => late.height,
        },
        scale: lerp(a.scale, b.scale, t),
        transition: late.transition,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{StateFlags, THEME};

    #[test]
    fn the_ends_are_the_two_styles() {
        let base = THEME.resolve(&THEME.tile, StateFlags::NONE);
        let focus = THEME.resolve(&THEME.tile, StateFlags::FOCUS);
        let start = mix(&base, &focus, 0.0);
        assert_eq!(start.scale, 1.0);
        assert_eq!(
            start.outline.map(|o| o.color.a),
            Some(0),
            "the ring is invisible at the start"
        );
        let end = mix(&base, &focus, 1.0);
        assert_eq!(end.scale, 1.12);
        assert_eq!(end.outline, focus.outline);
        assert_eq!(end.box_shadow, focus.box_shadow);
    }

    #[test]
    fn halfway_is_in_between() {
        let base = THEME.resolve(&THEME.tile, StateFlags::NONE);
        let focus = THEME.resolve(&THEME.tile, StateFlags::FOCUS);
        let half = mix(&base, &focus, 0.5);
        assert!((half.scale - 1.06).abs() < 1e-5);
        let ring = half.outline.unwrap();
        assert!((i32::from(ring.color.a) - 128).abs() <= 1);
        assert_eq!(ring.width, 2.0);
    }

    #[test]
    fn solid_backgrounds_blend() {
        let base = THEME.resolve(&THEME.editor_row, StateFlags::NONE);
        let focus = THEME.resolve(&THEME.editor_row, StateFlags::FOCUS);
        let (Some(Paint::Solid(a)), Some(Paint::Solid(b))) = (base.background, focus.background)
        else {
            panic!()
        };
        let Some(Paint::Solid(m)) = mix(&base, &focus, 0.5).background else {
            panic!()
        };
        assert_eq!(m, mix_color(a, b, 0.5));
        assert!(m.r > a.r.min(b.r) - 1 && m.r < a.r.max(b.r) + 1);
    }

    #[test]
    fn progress_is_clamped() {
        let base = THEME.resolve(&THEME.tile, StateFlags::NONE);
        let focus = THEME.resolve(&THEME.tile, StateFlags::FOCUS);
        assert_eq!(mix(&base, &focus, -3.0).scale, 1.0);
        assert_eq!(mix(&base, &focus, 7.0).scale, 1.12);
    }

    #[test]
    fn mixing_a_style_with_itself_changes_nothing() {
        let style = THEME.resolve(&THEME.button, StateFlags::NONE);
        assert_eq!(mix(&style, &style, 0.37), style);
    }
}
