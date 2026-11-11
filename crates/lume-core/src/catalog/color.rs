//! Tile colors: dominant color of an app icon, adjusted so white text stays readable.

use crate::theme::Color;

/// Minimum contrast ratio of white text on a tile (WCAG AA for normal text).
pub const MIN_CONTRAST: f32 = 4.5;

/// Width and height of the icon bitmap the platform bridge hands over.
pub const ICON_SIZE: usize = 24;

fn linear(channel: u8) -> f32 {
    let c = f32::from(channel) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Relative luminance of a color, from 0.0 (black) to 1.0 (white).
pub fn luminance(color: Color) -> f32 {
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

/// Contrast ratio of white text on `background`, from 1.0 to 21.0.
pub fn contrast_with_white(background: Color) -> f32 {
    1.05 / (luminance(background) + 0.05)
}

/// Darkens `color`, keeping its hue, until white text on it has at least [`MIN_CONTRAST`].
pub fn ensure_white_text_contrast(color: Color) -> Color {
    let mut current = Color { a: 255, ..color };
    while contrast_with_white(current) < MIN_CONTRAST {
        let darker = |c: u8| (f32::from(c) * 0.94).floor() as u8;
        current = Color {
            r: darker(current.r),
            g: darker(current.g),
            b: darker(current.b),
            a: 255,
        };
    }
    current
}

/// The dominant color of an icon given as ARGB pixels (`0xAARRGGBB`), or `None` when the icon
/// has no usable color (fully transparent, or only white, black and grays).
///
/// Pixels are grouped into 4096 color bins; colorful bins are favored over gray ones, and
/// the winner's average color is returned.
pub fn dominant_color(pixels: &[u32]) -> Option<Color> {
    #[derive(Clone, Copy, Default)]
    struct Bin {
        count: u32,
        weight: f32,
        r: u32,
        g: u32,
        b: u32,
    }
    let mut bins = vec![Bin::default(); 4096];
    for &pixel in pixels {
        let (a, r, g, b) = (
            (pixel >> 24) as u8,
            (pixel >> 16) as u8,
            (pixel >> 8) as u8,
            pixel as u8,
        );
        if a < 128 {
            continue;
        }
        let (max, min) = (r.max(g).max(b), r.min(g).min(b));
        // Near-white and near-black pixels are usually the icon's background or outline.
        if min > 235 || max < 24 {
            continue;
        }
        let saturation = f32::from(max - min) / f32::from(max);
        if saturation < 0.12 {
            continue;
        }
        let key = (usize::from(r >> 4) << 8) | (usize::from(g >> 4) << 4) | usize::from(b >> 4);
        let bin = &mut bins[key];
        bin.count += 1;
        bin.weight += 1.0 + 3.0 * saturation;
        bin.r += u32::from(r);
        bin.g += u32::from(g);
        bin.b += u32::from(b);
    }
    let best = bins
        .iter()
        .filter(|b| b.count > 0)
        .max_by(|a, b| a.weight.total_cmp(&b.weight))?;
    Some(Color::rgb(
        (best.r / best.count) as u8,
        (best.g / best.count) as u8,
        (best.b / best.count) as u8,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
        u32::from_be_bytes([a, r, g, b])
    }

    #[test]
    fn contrast_of_the_extremes() {
        assert!((contrast_with_white(Color::rgb(0, 0, 0)) - 21.0).abs() < 0.01);
        assert!((contrast_with_white(Color::rgb(255, 255, 255)) - 1.0).abs() < 0.01);
        // The mockup's Netflix red already reads well with white text.
        assert!(contrast_with_white(Color::rgb(0xB3, 0x12, 0x1C)) > 6.0);
    }

    #[test]
    fn dark_enough_colors_are_left_alone() {
        let color = Color::rgb(0x14, 0x5A, 0x96);
        assert_eq!(ensure_white_text_contrast(color), color);
    }

    #[test]
    fn light_colors_are_darkened_until_the_text_is_readable() {
        for color in [
            Color::rgb(255, 255, 0),
            Color::rgb(255, 255, 255),
            Color::rgb(200, 255, 200),
            Color::rgb(255, 220, 100),
            Color::rgb(0, 255, 255),
            Color::rgb(180, 180, 180),
        ] {
            let fixed = ensure_white_text_contrast(color);
            assert!(
                contrast_with_white(fixed) >= MIN_CONTRAST,
                "{color:?} -> {fixed:?}"
            );
            // Only darkened: no channel grew.
            assert!(fixed.r <= color.r && fixed.g <= color.g && fixed.b <= color.b);
        }
    }

    #[test]
    fn every_color_reaches_the_minimum_contrast() {
        // A coarse sweep of the whole RGB cube.
        for r in (0..=255).step_by(15) {
            for g in (0..=255).step_by(15) {
                for b in (0..=255).step_by(15) {
                    let fixed = ensure_white_text_contrast(Color::rgb(r as u8, g as u8, b as u8));
                    assert!(
                        contrast_with_white(fixed) >= MIN_CONTRAST,
                        "({r},{g},{b}) -> {fixed:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn the_hue_is_kept_when_darkening() {
        let fixed = ensure_white_text_contrast(Color::rgb(255, 128, 0));
        // Orange stays orange: red dominates, green is about half of it, blue is zero.
        assert!(fixed.r > fixed.g && fixed.g > fixed.b);
        assert!((f32::from(fixed.g) / f32::from(fixed.r) - 0.5).abs() < 0.05);
    }

    #[test]
    fn alpha_is_ignored_and_reset() {
        assert_eq!(
            ensure_white_text_contrast(Color {
                r: 10,
                g: 20,
                b: 30,
                a: 7
            })
            .a,
            255
        );
    }

    #[test]
    fn dominant_color_picks_the_main_hue() {
        let mut icon = vec![argb(255, 30, 120, 220); 300];
        icon.extend(vec![argb(255, 220, 40, 40); 100]);
        icon.extend(vec![argb(255, 255, 255, 255); 176]);
        let color = dominant_color(&icon).unwrap();
        assert!(color.b > 180 && color.r < 60, "{color:?}");
    }

    #[test]
    fn dominant_color_prefers_colorful_over_dull_pixels() {
        let mut icon = vec![argb(255, 120, 124, 126); 400];
        icon.extend(vec![argb(255, 20, 160, 90); 100]);
        let color = dominant_color(&icon).unwrap();
        assert!(
            color.g > color.r + 50,
            "the gray pixels were ignored: {color:?}"
        );
    }

    #[test]
    fn dominant_color_skips_transparent_white_and_black_pixels() {
        assert_eq!(dominant_color(&[]), None);
        assert_eq!(dominant_color(&vec![argb(0, 255, 0, 0); 100]), None);
        assert_eq!(dominant_color(&vec![argb(255, 255, 255, 255); 100]), None);
        assert_eq!(dominant_color(&vec![argb(255, 0, 0, 0); 100]), None);
        assert_eq!(dominant_color(&vec![argb(255, 128, 128, 128); 100]), None);
        let mixed = [
            argb(0, 255, 0, 0),
            argb(255, 250, 250, 250),
            argb(255, 200, 30, 30),
        ];
        assert!(dominant_color(&mixed).unwrap().r > 190);
    }

    #[test]
    fn dominant_color_averages_within_the_winning_bin() {
        let icon = [argb(255, 200, 20, 20), argb(255, 204, 22, 18)];
        assert_eq!(dominant_color(&icon), Some(Color::rgb(202, 21, 19)));
    }

    #[test]
    fn a_full_icon_bitmap_is_handled() {
        let icon = vec![argb(255, 60, 90, 200); ICON_SIZE * ICON_SIZE];
        assert_eq!(dominant_color(&icon), Some(Color::rgb(60, 90, 200)));
    }
}
