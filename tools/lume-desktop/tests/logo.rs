//! Compares the logo drawn by lume with the design file, pixel by pixel.

use lume_core::logo;
use lume_core::render::{Backend, DisplayList, Frame, Gles2Backend, Rect, Viewport};
use lume_core::theme::vars;
use lume_desktop::gl::HeadlessGl;
use std::path::Path;

struct Image {
    width: usize,
    rgba: Vec<u8>,
}

fn load(path: &Path) -> Option<Image> {
    let file = std::fs::File::open(path).ok()?;
    let mut reader = png::Decoder::new(std::io::BufReader::new(file))
        .read_info()
        .ok()?;
    let mut buffer = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buffer).ok()?;
    assert_eq!(info.color_type, png::ColorType::Rgba);
    Some(Image {
        width: info.width as usize,
        rgba: buffer,
    })
}

#[test]
fn the_mark_matches_the_design_file() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../mockup/Logo · Lume@2x.png");
    let Some(design) = load(&path) else {
        eprintln!("skipped: the design file is not available");
        return;
    };
    // The big mark in the design occupies (392, 271)-(728, 606) of the @2x image: 336 px.
    const SIZE: u32 = 336;
    let (origin_x, origin_y) = (392usize, 271usize);

    let Ok(gl) = HeadlessGl::new(SIZE, SIZE) else {
        eprintln!("skipped: no GL context");
        return;
    };
    let mut backend = Gles2Backend::new(gl.glow()).expect("shaders");
    let mut list = DisplayList::new();
    // 168 logical units at 2 pixels each.
    logo::draw_mark(
        &mut list,
        Rect::new(0.0, 0.0, 168.0, 168.0),
        vars::ACCENT,
        vars::BG,
    );
    let viewport = Viewport {
        width: SIZE,
        height: SIZE,
        scale: 2.0,
        offset_x: 0.0,
        offset_y: 0.0,
        edge_softness: lume_core::theme::vars::EDGE_SOFTNESS,
    };
    backend.render(&viewport, vars::BG, &Frame::build(&list));
    let rendered = backend.read_pixels(SIZE, SIZE);
    backend.destroy();

    // Set LUME_LOGO_DIFF=<dir> to write our rendering and a difference image for inspection.
    let debug_dir = std::env::var_os("LUME_LOGO_DIFF").map(std::path::PathBuf::from);
    let mut diff_image = vec![0u8; (SIZE * SIZE * 4) as usize];
    // A pixel counts as different only when no design pixel within 2 device pixels
    // (1 logical unit, half the tolerance of the theming spec) has a similar color.
    const REACH: i32 = 2;
    let design_pixel = |x: i32, y: i32| -> Option<&[u8]> {
        let (x, y) = (
            x.checked_add(origin_x as i32)?,
            y.checked_add(origin_y as i32)?,
        );
        (x >= 0 && y >= 0)
            .then(|| &design.rgba[(y as usize * design.width + x as usize) * 4..][..3])
    };
    let mut different = 0;
    let mut worst = 0;
    for y in 0..SIZE as i32 {
        for x in 0..SIZE as i32 {
            let ours = &rendered[(y as usize * SIZE as usize + x as usize) * 4..][..3];
            let mut best = u8::MAX;
            for dy in -REACH..=REACH {
                for dx in -REACH..=REACH {
                    if let Some(theirs) = design_pixel(x + dx, y + dy) {
                        let delta = ours
                            .iter()
                            .zip(theirs)
                            .map(|(a, b)| a.abs_diff(*b))
                            .max()
                            .unwrap();
                        best = best.min(delta);
                    }
                }
            }
            worst = worst.max(best);
            let pixel = &mut diff_image[(y as usize * SIZE as usize + x as usize) * 4..][..4];
            if best > 48 {
                different += 1;
                pixel.copy_from_slice(&[255, 0, 0, 255]);
            } else {
                pixel.copy_from_slice(&[0, 0, 0, 255]);
            }
        }
    }
    if let Some(dir) = debug_dir {
        lume_desktop::png_out::write_rgba(&dir.join("logo-ours.png"), SIZE, SIZE, &rendered)
            .unwrap();
        lume_desktop::png_out::write_rgba(&dir.join("logo-diff.png"), SIZE, SIZE, &diff_image)
            .unwrap();
    }
    let total = (SIZE * SIZE) as f32;
    let fraction = different as f32 / total;
    eprintln!(
        "logo: {different} of {total} pixels differ ({:.2}%), worst channel delta {worst}",
        fraction * 100.0
    );
    assert!(
        fraction < 0.01,
        "{:.2}% of the logo pixels differ from the design",
        fraction * 100.0
    );
}
