//! App icons: bounded PNG decode and bilinear scaling into the atlas cells.
//!
//! webOS apps ship their icon as a PNG in the app folder. HOME scales it to fit a cell
//! (tile or dock), centered over transparency, premultiplied as the atlas expects.

use std::path::Path;

/// Source files over this size or resolution are rejected.
const MAX_FILE: u64 = 1024 * 1024;
const MAX_SIDE: u32 = 2048;

pub struct Decoded {
    pub width: u32,
    pub height: u32,
    /// Straight (not premultiplied) RGBA.
    pub rgba: Vec<u8>,
}

/// Decodes a PNG file into RGBA8.
pub fn decode_png(path: &Path) -> Option<Decoded> {
    let file = std::fs::File::open(path).ok()?;
    if file.metadata().ok()?.len() > MAX_FILE {
        return None;
    }
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let (width, height) = (reader.info().width, reader.info().height);
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return None;
    }
    let mut buffer = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buffer).ok()?;
    buffer.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buffer,
        png::ColorType::Rgb => buffer
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::Grayscale => buffer.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::GrayscaleAlpha => buffer
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Indexed => return None,
    };
    (rgba.len() == (width * height * 4) as usize).then_some(Decoded {
        width,
        height,
        rgba,
    })
}

/// Samples `source` at (`x`, `y`) in its own pixel space, bilinearly, straight RGBA.
fn sample(source: &Decoded, x: f32, y: f32) -> [f32; 4] {
    let x = x.clamp(0.0, source.width as f32 - 1.0);
    let y = y.clamp(0.0, source.height as f32 - 1.0);
    let (x0, y0) = (x.floor() as u32, y.floor() as u32);
    let (x1, y1) = (
        (x0 + 1).min(source.width - 1),
        (y0 + 1).min(source.height - 1),
    );
    let (fx, fy) = (x - x0 as f32, y - y0 as f32);
    let at = |px: u32, py: u32| {
        let i = ((py * source.width + px) * 4) as usize;
        [
            source.rgba[i] as f32,
            source.rgba[i + 1] as f32,
            source.rgba[i + 2] as f32,
            source.rgba[i + 3] as f32,
        ]
    };
    let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
    std::array::from_fn(|i| {
        (a[i] * (1.0 - fx) + b[i] * fx) * (1.0 - fy) + (c[i] * (1.0 - fx) + d[i] * fx) * fy
    })
}

/// Scales the icon to fit a `width` x `height` cell, centered over transparency, and returns
/// the cell as premultiplied RGBA (what the icon atlas expects).
pub fn fit_cell(source: &Decoded, width: u32, height: u32) -> Vec<u8> {
    let mut cell = vec![0u8; (width * height * 4) as usize];
    let scale = (width as f32 / source.width as f32)
        .min(height as f32 / source.height as f32)
        .min(f32::INFINITY);
    let (out_w, out_h) = (
        (source.width as f32 * scale).round().max(1.0) as u32,
        (source.height as f32 * scale).round().max(1.0) as u32,
    );
    let (left, top) = (
        (width - out_w.min(width)) / 2,
        (height - out_h.min(height)) / 2,
    );
    for y in 0..out_h.min(height) {
        for x in 0..out_w.min(width) {
            // The source point of this output pixel's center.
            let sx = (x as f32 + 0.5) / scale - 0.5;
            let sy = (y as f32 + 0.5) / scale - 0.5;
            let [r, g, b, a] = sample(source, sx, sy);
            let i = (((top + y) * width + (left + x)) * 4) as usize;
            let alpha = a / 255.0;
            cell[i] = (r * alpha).round() as u8;
            cell[i + 1] = (g * alpha).round() as u8;
            cell[i + 2] = (b * alpha).round() as u8;
            cell[i + 3] = a.round() as u8;
        }
    }
    cell
}

/// Whether the icon is a glyph on transparency (LG's system apps) rather than a full-bleed
/// logo: tinting the tile with a glyph's own color would hide the glyph.
pub fn is_glyph(source: &Decoded) -> bool {
    let opaque = source.rgba.chunks_exact(4).filter(|p| p[3] >= 128).count();
    opaque * 2 < source.rgba.len() / 4
}

/// A small ARGB thumbnail for deriving the tile color (see `catalog::color`).
pub fn color_thumbnail(source: &Decoded, side: usize) -> Vec<u32> {
    let mut pixels = Vec::with_capacity(side * side);
    for y in 0..side {
        for x in 0..side {
            let sx = (x as f32 + 0.5) / side as f32 * source.width as f32 - 0.5;
            let sy = (y as f32 + 0.5) / side as f32 * source.height as f32 - 0.5;
            let [r, g, b, a] = sample(source, sx, sy);
            pixels.push(
                (a.round() as u32) << 24
                    | (r.round() as u32) << 16
                    | (g.round() as u32) << 8
                    | b.round() as u32,
            );
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Decoded {
        Decoded {
            width,
            height,
            rgba: rgba
                .iter()
                .copied()
                .cycle()
                .take((width * height * 4) as usize)
                .collect(),
        }
    }

    #[test]
    fn a_wide_icon_is_centered_vertically_and_premultiplied() {
        let source = solid(100, 50, [200, 100, 0, 128]);
        let cell = fit_cell(&source, 40, 40);
        assert_eq!(cell.len(), 40 * 40 * 4);
        // The scaled icon is 40x20, centered: rows 0..10 and 30..40 stay transparent.
        assert_eq!(&cell[0..4], [0, 0, 0, 0]);
        let center = ((20 * 40 + 20) * 4) as usize;
        let premultiplied = (200.0_f32 * 128.0 / 255.0).round() as u8;
        assert_eq!(cell[center], premultiplied);
        assert_eq!(cell[center + 3], 128);
    }

    #[test]
    fn a_mostly_transparent_icon_is_a_glyph() {
        let mut glyph = solid(10, 10, [0, 0, 0, 0]);
        for p in glyph.rgba.chunks_exact_mut(4).take(30) {
            p.copy_from_slice(&[120, 60, 200, 255]);
        }
        assert!(is_glyph(&glyph));
        assert!(!is_glyph(&solid(10, 10, [120, 60, 200, 255])));
    }

    #[test]
    fn the_color_thumbnail_has_the_requested_size() {
        let source = solid(80, 80, [10, 20, 30, 255]);
        let pixels = color_thumbnail(&source, 24);
        assert_eq!(pixels.len(), 24 * 24);
        assert_eq!(pixels[0], 0xFF0A141E);
    }
}
