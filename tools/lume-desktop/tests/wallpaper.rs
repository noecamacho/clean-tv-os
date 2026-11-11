//! The wallpaper as the real OpenGL ES 2.0 backend draws it. Tests are skipped when no EGL
//! implementation is available.

use lume_core::app::Painter;
use lume_desktop::screenshot;

fn wallpaper_frame(width: u32, height: u32) -> Option<Vec<u8>> {
    wallpaper_frame_of(lume_core::wallpaper::WallpaperId::default(), width, height)
}

fn wallpaper_frame_of(
    id: lume_core::wallpaper::WallpaperId,
    width: u32,
    height: u32,
) -> Option<Vec<u8>> {
    let result = screenshot::render(width, height, |scene| {
        let mut painter = Painter {
            list: scene.list,
            text: scene.text,
            display: *scene.display,
        };
        painter.wallpaper_at(id, false, 1.0);
    });
    match result {
        Ok(pixels) => Some(pixels),
        Err(e) if e.contains("EGL") || e.contains("GL context") => {
            eprintln!("skipped: {e}");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn at_1080p_it_matches_the_catalog_image_within_1_in_255() {
    let Some(pixels) = wallpaper_frame(1920, 1080) else {
        return;
    };
    let rgb = lume_core::wallpaper::current().decompress_rgb();
    assert_eq!(rgb.len(), 1920 * 1080 * 3);
    let mut total = 0u64;
    for p in 0..(1920 * 1080) as usize {
        for c in 0..3 {
            total += u64::from(pixels[p * 4 + c].abs_diff(rgb[p * 3 + c]));
        }
    }
    let mean = total as f64 / (1920.0 * 1080.0 * 3.0);
    assert!(mean <= 1.0, "mean absolute difference {mean:.2}/255");
}

/// The dimmed wallpaper of the text screens, rendered as it is and without its dither.
fn dimmed_with_and_without_dither(width: u32, height: u32) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut session = match screenshot::Session::new(width, height) {
        Ok(session) => session,
        Err(e) if e.contains("EGL") || e.contains("GL context") => {
            eprintln!("skipped: {e}");
            return None;
        }
        Err(e) => panic!("{e}"),
    };
    session.build(|scene| {
        let mut painter = Painter {
            list: scene.list,
            text: scene.text,
            display: *scene.display,
        };
        painter.wallpaper(true);
    });
    let dithered = session.render();
    session.set_wallpaper_dither(false);
    let plain = session.render();
    session.finish();
    Some((dithered, plain))
}

#[test]
fn the_dither_breaks_the_bands_without_changing_the_average_color() {
    for (width, height) in [(1280, 720), (1920, 1080)] {
        let Some((dithered, plain)) = dimmed_with_and_without_dither(width, height) else {
            return;
        };
        assert_ne!(
            dithered, plain,
            "{width}x{height}: the wallpaper is dithered"
        );
        let mut worst = 0.0f64;
        for top in (0..height - 15).step_by(16) {
            for left in (0..width - 15).step_by(16) {
                for c in 0..3 {
                    let (mut a, mut b) = (0u32, 0u32);
                    for y in top..top + 16 {
                        for x in left..left + 16 {
                            let i = ((y * width + x) * 4) as usize + c;
                            a += u32::from(dithered[i]);
                            b += u32::from(plain[i]);
                        }
                    }
                    worst = worst.max((f64::from(a) - f64::from(b)).abs() / 256.0);
                }
            }
        }
        assert!(
            worst <= 1.0,
            "{width}x{height}: a 16x16 region changed {worst:.2}/255 on average"
        );
    }
}

/// Lanczos-3 resampling of an RGB image (the reference the GPU's reduction is compared with).
fn lanczos_resize(src: &[u8], (sw, sh): (usize, usize), (dw, dh): (usize, usize)) -> Vec<f32> {
    fn kernel(x: f32) -> f32 {
        if x.abs() >= 3.0 {
            return 0.0;
        }
        if x == 0.0 {
            return 1.0;
        }
        let px = std::f32::consts::PI * x;
        3.0 * px.sin() * (px / 3.0).sin() / (px * px)
    }
    // Weights of every destination index over the source indices.
    let weights = |from: usize, to: usize| -> Vec<(usize, Vec<f32>)> {
        let scale = from as f32 / to as f32;
        let support = 3.0 * scale.max(1.0);
        (0..to)
            .map(|d| {
                let center = (d as f32 + 0.5) * scale;
                let first = ((center - support).floor().max(0.0)) as usize;
                let last = ((center + support).ceil() as usize).min(from);
                let mut w: Vec<f32> = (first..last)
                    .map(|s| kernel((s as f32 + 0.5 - center) / scale.max(1.0)))
                    .collect();
                let total: f32 = w.iter().sum();
                w.iter_mut().for_each(|v| *v /= total);
                (first, w)
            })
            .collect()
    };
    let (wx, wy) = (weights(sw, dw), weights(sh, dh));
    let mut horizontal = vec![0.0f32; dw * sh * 3];
    for y in 0..sh {
        for (x, (first, w)) in wx.iter().enumerate() {
            for c in 0..3 {
                horizontal[(y * dw + x) * 3 + c] = w
                    .iter()
                    .enumerate()
                    .map(|(i, v)| v * f32::from(src[(y * sw + first + i) * 3 + c]))
                    .sum();
            }
        }
    }
    let mut out = vec![0.0f32; dw * dh * 3];
    for (y, (first, w)) in wy.iter().enumerate() {
        for x in 0..dw {
            for c in 0..3 {
                out[(y * dw + x) * 3 + c] = w
                    .iter()
                    .enumerate()
                    .map(|(i, v)| v * horizontal[((first + i) * dw + x) * 3 + c])
                    .sum();
            }
        }
    }
    out
}

#[test]
fn at_720p_it_matches_a_lanczos_reduction_of_the_catalog_image_within_2_in_255() {
    let Some(pixels) = wallpaper_frame(1280, 720) else {
        return;
    };
    let rgb = lume_core::wallpaper::current().decompress_rgb();
    let reference = lanczos_resize(&rgb, (1920, 1080), (1280, 720));
    let mut total = 0.0f64;
    for p in 0..1280 * 720 {
        for c in 0..3 {
            total += f64::from((f32::from(pixels[p * 4 + c]) - reference[p * 3 + c]).abs());
        }
    }
    let mean = total / (1280.0 * 720.0 * 3.0);
    assert!(mean <= 2.0, "mean absolute difference {mean:.2}/255");
}

/// Energy of the fine detail: the mean square of the discrete Laplacian of the luminance.
fn fine_detail_energy(luma: &[f32], width: usize, height: usize) -> f64 {
    let mut total = 0.0f64;
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let at = |dx: isize, dy: isize| {
                luma[(y as isize + dy) as usize * width + (x as isize + dx) as usize]
            };
            let laplacian = 4.0 * at(0, 0) - at(-1, 0) - at(1, 0) - at(0, -1) - at(0, 1);
            total += f64::from(laplacian * laplacian);
        }
    }
    total / ((width - 2) * (height - 2)) as f64
}

#[test]
fn thin_lines_do_not_gain_moire_when_reduced_to_720p() {
    use lume_core::wallpaper::WallpaperId;
    for id in [WallpaperId::SynthwaveGrid, WallpaperId::GlassFacade] {
        let Some(pixels) = wallpaper_frame_of(id, 1280, 720) else {
            return;
        };
        let rgb = id.wallpaper().decompress_rgb();
        let reference = lanczos_resize(&rgb, (1920, 1080), (1280, 720));
        let to_luma = |f: &dyn Fn(usize, usize) -> f32| -> Vec<f32> {
            (0..1280 * 720)
                .map(|p| 0.2126 * f(p, 0) + 0.7152 * f(p, 1) + 0.0722 * f(p, 2))
                .collect()
        };
        let rendered = to_luma(&|p, c| f32::from(pixels[p * 4 + c]));
        let expected = to_luma(&|p, c| reference[p * 3 + c]);
        let (a, b) = (
            fine_detail_energy(&rendered, 1280, 720),
            fine_detail_energy(&expected, 1280, 720),
        );
        // The dither adds a little energy of its own, so the limit is the reference plus 10%.
        assert!(
            a <= b * 1.10 + 0.5,
            "{}: fine detail energy {a:.3} against {b:.3} of the Lanczos reduction",
            id.as_str()
        );
    }
}

#[test]
fn smooth_gradients_have_no_long_runs_of_one_color_at_720p_and_1080p() {
    // Without dither a dark gradient shows flat bands tens of pixels wide; the dither breaks them.
    for (width, height) in [(1280usize, 720usize), (1920, 1080)] {
        let Some(pixels) = wallpaper_frame(width as u32, height as u32) else {
            return;
        };
        let mut longest = 0;
        for y in 0..height {
            let mut run = 1;
            for x in 1..width {
                let (a, b) = ((y * width + x - 1) * 4, (y * width + x) * 4);
                if pixels[a..a + 3] == pixels[b..b + 3] {
                    run += 1;
                    longest = longest.max(run);
                } else {
                    run = 1;
                }
            }
        }
        assert!(
            longest <= 24,
            "{width}x{height}: a run of {longest} identical pixels"
        );
    }
}
