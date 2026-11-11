//! Renders frames through the real OpenGL ES 2.0 backend (Mesa software rendering on CI)
//! and checks the pixels. Tests are skipped when no EGL implementation is available.

use lume_core::render::{Backend, DisplayList, Frame, Gles2Backend, Rect, Viewport};
use lume_core::theme::{Color, Gradient, GradientStop, Paint};
use lume_desktop::gl::HeadlessGl;

const W: u32 = 400;
const H: u32 = 300;
const BG: Color = Color::rgb(16, 24, 32);

struct Rig {
    // Declaration order matters: the backend must be dropped before the GL context.
    backend: Gles2Backend,
    _gl: HeadlessGl,
}

fn rig() -> Option<Rig> {
    let gl = match HeadlessGl::new(W, H) {
        Ok(gl) => gl,
        Err(e) => {
            eprintln!("skipped: no GL context ({e})");
            return None;
        }
    };
    let backend = Gles2Backend::new(gl.glow()).expect("the shaders must compile and link");
    Some(Rig { backend, _gl: gl })
}

fn render(rig: &mut Rig, list: &DisplayList) -> Pixels {
    render_with(
        rig,
        list,
        Viewport {
            width: W,
            height: H,
            scale: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            edge_softness: lume_core::theme::vars::EDGE_SOFTNESS,
        },
    )
}

fn render_with(rig: &mut Rig, list: &DisplayList, viewport: Viewport) -> Pixels {
    rig.backend.render(&viewport, BG, &Frame::build(list));
    Pixels(rig.backend.read_pixels(W, H))
}

struct Pixels(Vec<u8>);

impl Pixels {
    fn at(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * W + x) * 4) as usize;
        [self.0[i], self.0[i + 1], self.0[i + 2], self.0[i + 3]]
    }
}

fn close(actual: [u8; 4], expected: [u8; 4], tolerance: u8) -> bool {
    actual
        .iter()
        .zip(expected)
        .all(|(a, e)| a.abs_diff(e) <= tolerance)
}

#[test]
fn clear_color_fills_the_surface() {
    let Some(mut rig) = rig() else { return };
    let pixels = render(&mut rig, &DisplayList::new());
    assert_eq!(pixels.at(0, 0), [16, 24, 32, 255]);
    assert_eq!(pixels.at(W - 1, H - 1), [16, 24, 32, 255]);
}

#[test]
fn solid_rounded_rect_has_antialiased_round_corners() {
    let Some(mut rig) = rig() else { return };
    let mut list = DisplayList::new();
    list.fill(
        Rect::new(50.0, 50.0, 200.0, 100.0),
        20.0,
        Paint::Solid(Color::rgb(200, 30, 40)),
    );
    let pixels = render(&mut rig, &list);
    assert!(
        close(pixels.at(150, 100), [200, 30, 40, 255], 1),
        "center is {:?}",
        pixels.at(150, 100)
    );
    // The very corner pixel is outside the rounded shape.
    assert_eq!(pixels.at(50, 50), [16, 24, 32, 255]);
    // A straight edge on a pixel boundary spreads over the theme's --edge-softness: the pixel
    // inside and the one outside share it, and the next ones are untouched.
    let inside = 0.5 + 0.5 / lume_core::theme::vars::EDGE_SOFTNESS;
    let mix = |cover: f32| {
        let blend =
            |fg: u8, bg: u8| (f32::from(fg) * cover + f32::from(bg) * (1.0 - cover)).round() as u8;
        [blend(200, 16), blend(30, 24), blend(40, 32), 255]
    };
    assert!(close(pixels.at(150, 51), [200, 30, 40, 255], 1));
    assert!(
        close(pixels.at(150, 50), mix(inside), 3),
        "{:?}",
        pixels.at(150, 50)
    );
    assert!(
        close(pixels.at(150, 49), mix(1.0 - inside), 3),
        "{:?}",
        pixels.at(150, 49)
    );
    assert_eq!(pixels.at(150, 48), [16, 24, 32, 255]);
    // The corner arc is partially covered: some pixel on the diagonal is a blend.
    let blended = (0..20)
        .flat_map(|dy| (0..20).map(move |dx| (dx, dy)))
        .any(|(dx, dy)| {
            let p = pixels.at(50 + dx, 50 + dy);
            p != [16, 24, 32, 255] && !close(p, [200, 30, 40, 255], 4)
        });
    assert!(blended, "the corner should be antialiased");
}

#[test]
fn translucent_fill_blends_with_the_background() {
    let Some(mut rig) = rig() else { return };
    let mut list = DisplayList::new();
    list.fill(
        Rect::new(0.0, 0.0, 100.0, 100.0),
        0.0,
        Paint::Solid(Color {
            r: 255,
            g: 255,
            b: 255,
            a: 128,
        }),
    );
    let pixels = render(&mut rig, &list);
    // 50% white over (16, 24, 32).
    assert!(
        close(pixels.at(50, 50), [135, 140, 144, 255], 3),
        "got {:?}",
        pixels.at(50, 50)
    );
}

#[test]
fn linear_gradient_runs_left_to_right() {
    const STOPS: [GradientStop; 2] = [
        GradientStop {
            color: Color::rgb(0, 0, 0),
            position: 0.0,
        },
        GradientStop {
            color: Color::rgb(255, 255, 255),
            position: 1.0,
        },
    ];
    let Some(mut rig) = rig() else { return };
    let mut list = DisplayList::new();
    list.fill(
        Rect::new(0.0, 0.0, 300.0, 100.0),
        0.0,
        Paint::Gradient(Gradient::new(90.0, &STOPS)),
    );
    let pixels = render(&mut rig, &list);
    let left = pixels.at(2, 50)[0];
    let middle = pixels.at(150, 50)[0];
    let right = pixels.at(297, 50)[0];
    assert!(left < 10, "left is {left}");
    assert!(right > 245, "right is {right}");
    assert!((i32::from(middle) - 128).abs() < 6, "middle is {middle}");
    // Constant along the vertical axis.
    assert!(pixels.at(150, 5)[0].abs_diff(pixels.at(150, 95)[0]) <= 1);
}

#[test]
fn three_stop_gradient_follows_the_stop_positions() {
    const STOPS: [GradientStop; 3] = [
        GradientStop {
            color: Color::rgb(255, 0, 0),
            position: 0.0,
        },
        GradientStop {
            color: Color::rgb(0, 255, 0),
            position: 0.25,
        },
        GradientStop {
            color: Color::rgb(0, 0, 255),
            position: 1.0,
        },
    ];
    let Some(mut rig) = rig() else { return };
    let mut list = DisplayList::new();
    list.fill(
        Rect::new(0.0, 0.0, 400.0, 100.0),
        0.0,
        Paint::Gradient(Gradient::new(90.0, &STOPS)),
    );
    let pixels = render(&mut rig, &list);
    // At 25% (x = 100) the color is pure green.
    assert!(
        close(pixels.at(100, 50), [0, 255, 0, 255], 8),
        "got {:?}",
        pixels.at(100, 50)
    );
    // At 62.5% (x = 250) it is halfway between green and blue.
    assert!(
        close(pixels.at(250, 50), [0, 128, 128, 255], 8),
        "got {:?}",
        pixels.at(250, 50)
    );
}

#[test]
fn ring_draws_only_the_border() {
    let Some(mut rig) = rig() else { return };
    let mut list = DisplayList::new();
    list.ring(
        Rect::new(100.0, 100.0, 120.0, 80.0),
        12.0,
        4.0,
        Color::WHITE,
    );
    let pixels = render(&mut rig, &list);
    assert!(
        close(pixels.at(160, 101), [255, 255, 255, 255], 2),
        "top edge: {:?}",
        pixels.at(160, 101)
    );
    assert!(close(pixels.at(101, 140), [255, 255, 255, 255], 2));
    // The inside is untouched.
    assert_eq!(pixels.at(160, 140), [16, 24, 32, 255]);
    assert_eq!(pixels.at(160, 96), [16, 24, 32, 255]);
}

#[test]
fn shadow_is_soft_and_extends_past_the_shape() {
    let Some(mut rig) = rig() else { return };
    let mut list = DisplayList::new();
    list.shadow(
        Rect::new(100.0, 100.0, 100.0, 60.0),
        8.0,
        Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        },
        24.0,
    );
    let pixels = render(&mut rig, &list);
    let inside = pixels.at(150, 130)[0];
    let edge = pixels.at(150, 100)[0];
    let outside = pixels.at(150, 92)[0];
    let far = pixels.at(150, 40)[0];
    assert!(
        inside > edge && edge > outside && outside > far,
        "{inside} {edge} {outside} {far}"
    );
    assert_eq!(far, 16, "far away the background is untouched");
}

#[test]
fn clip_limits_the_drawing() {
    let Some(mut rig) = rig() else { return };
    let mut list = DisplayList::new();
    list.push_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
    list.fill(
        Rect::new(50.0, 50.0, 200.0, 200.0),
        0.0,
        Paint::Solid(Color::rgb(255, 0, 0)),
    );
    list.pop_clip();
    list.fill(
        Rect::new(300.0, 10.0, 50.0, 50.0),
        0.0,
        Paint::Solid(Color::rgb(0, 255, 0)),
    );
    let pixels = render(&mut rig, &list);
    assert!(close(pixels.at(75, 75), [255, 0, 0, 255], 1));
    assert_eq!(pixels.at(150, 150), [16, 24, 32, 255]);
    assert!(close(pixels.at(320, 30), [0, 255, 0, 255], 1));
}

#[test]
fn viewport_scale_and_offset_place_the_canvas() {
    let Some(mut rig) = rig() else { return };
    let mut list = DisplayList::new();
    list.fill(
        Rect::new(10.0, 10.0, 20.0, 20.0),
        0.0,
        Paint::Solid(Color::rgb(255, 255, 0)),
    );
    let viewport = Viewport {
        width: W,
        height: H,
        scale: 2.0,
        offset_x: 100.0,
        offset_y: 50.0,
        edge_softness: lume_core::theme::vars::EDGE_SOFTNESS,
    };
    let pixels = render_with(&mut rig, &list, viewport);
    // Logical (10..30) becomes device 120..160 horizontally and 70..110 vertically.
    assert!(close(pixels.at(140, 90), [255, 255, 0, 255], 1));
    assert_eq!(pixels.at(115, 90), [16, 24, 32, 255]);
    assert_eq!(pixels.at(140, 65), [16, 24, 32, 255]);
}

#[test]
fn glyphs_sample_the_atlas_alpha_and_tint() {
    let Some(mut rig) = rig() else { return };
    // 2x1 atlas: left texel opaque, right texel transparent.
    rig.backend.set_atlas(2, 1, &[255, 0]);
    let mut list = DisplayList::new();
    list.glyph(
        Rect::new(10.0, 10.0, 20.0, 20.0),
        Rect::new(0.0, 0.0, 0.5, 1.0),
        Color::rgb(0, 200, 255),
    );
    list.glyph(
        Rect::new(50.0, 10.0, 20.0, 20.0),
        Rect::new(0.5, 0.0, 0.5, 1.0),
        Color::rgb(0, 200, 255),
    );
    let pixels = render(&mut rig, &list);
    // Pixel 19 maps left of the first texel's center, so linear filtering reads only that texel.
    assert!(
        close(pixels.at(19, 20), [0, 200, 255, 255], 2),
        "got {:?}",
        pixels.at(19, 20)
    );
    assert_eq!(pixels.at(60, 20), [16, 24, 32, 255]);
}

#[test]
fn batches_keep_painter_order_across_programs() {
    let Some(mut rig) = rig() else { return };
    rig.backend.set_atlas(1, 1, &[255]);
    let mut list = DisplayList::new();
    list.fill(
        Rect::new(0.0, 0.0, 100.0, 100.0),
        0.0,
        Paint::Solid(Color::rgb(255, 0, 0)),
    );
    list.glyph(
        Rect::new(20.0, 20.0, 40.0, 40.0),
        Rect::new(0.0, 0.0, 1.0, 1.0),
        Color::rgb(0, 0, 255),
    );
    list.fill(
        Rect::new(30.0, 30.0, 10.0, 10.0),
        0.0,
        Paint::Solid(Color::rgb(0, 255, 0)),
    );
    let pixels = render(&mut rig, &list);
    assert!(close(pixels.at(10, 10), [255, 0, 0, 255], 1));
    assert!(close(pixels.at(25, 25), [0, 0, 255, 255], 1));
    assert!(
        close(pixels.at(35, 35), [0, 255, 0, 255], 1),
        "the last rect must be on top"
    );
}

#[test]
fn atlas_regions_can_be_updated_in_place() {
    let Some(mut rig) = rig() else { return };
    rig.backend.set_atlas(4, 1, &[0, 0, 0, 0]);
    rig.backend.update_atlas([2, 0, 2, 1], &[255, 255]);
    assert_eq!(rig.backend.atlas_size(), Some((4, 1)));
    let mut list = DisplayList::new();
    list.glyph(
        Rect::new(0.0, 0.0, 40.0, 10.0),
        Rect::new(0.0, 0.0, 1.0, 1.0),
        Color::WHITE,
    );
    let pixels = render(&mut rig, &list);
    assert_eq!(pixels.at(5, 5), [16, 24, 32, 255], "the left half is empty");
    assert!(close(pixels.at(35, 5), [255, 255, 255, 255], 2));
}

#[test]
fn image_regions_are_filled_in_place_and_drawn() {
    let Some(mut rig) = rig() else { return };
    // An empty 4x2 texture; only the left 2x2 cell is filled, with opaque red.
    rig.backend.ensure_image(4, 2);
    let red = [255, 0, 0, 255].repeat(4);
    rig.backend.update_image([0, 0, 2, 2], &red);
    rig.backend.ensure_image(4, 2); // a second call at the same size keeps the texture
    let mut list = DisplayList::new();
    list.image(
        Rect::new(10.0, 10.0, 20.0, 20.0),
        Rect::new(0.0, 0.0, 0.5, 1.0),
        Color::WHITE,
    );
    let pixels = render(&mut rig, &list);
    assert!(
        close(pixels.at(20, 20), [255, 0, 0, 255], 2),
        "got {:?}",
        pixels.at(20, 20)
    );
    assert_eq!(pixels.at(60, 60), [16, 24, 32, 255], "outside the quad");

    // Another size (another scale's cells) makes the texture again, to be filled anew.
    rig.backend.ensure_image(8, 8);
    let green = [0, 255, 0, 255].repeat(16);
    rig.backend.update_image([4, 4, 4, 4], &green);
    let mut list = DisplayList::new();
    list.image(
        Rect::new(10.0, 10.0, 20.0, 20.0),
        Rect::new(0.5, 0.5, 0.5, 0.5),
        Color::WHITE,
    );
    let pixels = render(&mut rig, &list);
    assert!(
        close(pixels.at(20, 20), [0, 255, 0, 255], 2),
        "got {:?}",
        pixels.at(20, 20)
    );
}

#[test]
fn the_wallpaper_texture_is_created_when_first_drawn_and_freed_with_the_context() {
    let Some(mut rig) = rig() else { return };
    assert!(!rig.backend.has_wallpaper(), "nothing is created up front");

    // A frame without the wallpaper does not create it.
    let mut list = DisplayList::new();
    list.fill(
        Rect::new(0.0, 0.0, 10.0, 10.0),
        0.0,
        Paint::Solid(Color::rgb(1, 2, 3)),
    );
    render(&mut rig, &list);
    assert!(!rig.backend.has_wallpaper());

    // The whole texture over the whole surface: it replaces the clear color.
    let mut list = DisplayList::new();
    list.wallpaper(
        Rect::new(0.0, 0.0, W as f32, H as f32),
        Rect::new(0.0, 0.0, 1.0, 1.0),
        Color::WHITE,
    );
    let pixels = render(&mut rig, &list);
    assert!(rig.backend.has_wallpaper());
    let wallpaper = lume_core::wallpaper::current();
    let rgb = wallpaper.decompress_rgb();
    let texel = |x: usize, y: usize| {
        let i = (y * wallpaper.width as usize + x) * 3;
        [rgb[i], rgb[i + 1], rgb[i + 2], 255]
    };
    // Sample the middle of a pixel and compare with the same spot of the source image; the linear
    // filter blurs a little, so the tolerance is loose, but it must be the wallpaper and not BG.
    let (sx, sy) = (W / 2, H / 2);
    let expected = texel(
        (wallpaper.width / 2) as usize,
        (wallpaper.height / 2) as usize,
    );
    assert!(
        close(pixels.at(sx, sy), expected, 24),
        "got {:?}, expected about {expected:?}",
        pixels.at(sx, sy)
    );
    // Every pixel is covered: no leftover clear color at the corners' neighbourhood is required,
    // but the corners must come from the texture (clamped to the edge), not from BG.
    let corner = texel(0, 0);
    assert!(
        close(pixels.at(0, 0), corner, 24),
        "{:?} vs {corner:?}",
        pixels.at(0, 0)
    );

    // A second frame keeps the same texture; destroying the backend frees it with the rest.
    render(&mut rig, &list);
    assert!(rig.backend.has_wallpaper());
    let Rig { backend, _gl } = rig;
    backend.destroy();
}

#[test]
fn a_new_wallpaper_is_prepared_off_the_frame_and_only_the_last_choice_is_uploaded() {
    use lume_core::wallpaper::WallpaperId;
    let Some(mut rig) = rig() else { return };
    rig.backend.set_async_wallpaper(true);

    // With nothing to show yet, the first frame waits for the wallpaper: it is there at once.
    rig.backend.ensure_wallpaper(WallpaperId::Aurora);
    assert_eq!(rig.backend.current_wallpaper(), Some(WallpaperId::Aurora));

    // Choosing others in a row: the current one stays until the last choice is ready.
    for id in [
        WallpaperId::MountainStars,
        WallpaperId::MirrorLake,
        WallpaperId::GlassFacade,
    ] {
        rig.backend.ensure_wallpaper(id);
        assert_eq!(
            rig.backend.current_wallpaper(),
            Some(WallpaperId::Aurora),
            "the frame does not wait for {}",
            id.as_str()
        );
    }
    let mut seen = Vec::new();
    let started = std::time::Instant::now();
    while rig.backend.current_wallpaper() != Some(WallpaperId::GlassFacade) {
        assert!(started.elapsed().as_secs() < 10, "the wallpaper never came");
        rig.backend.ensure_wallpaper(WallpaperId::GlassFacade);
        seen.push(rig.backend.current_wallpaper());
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    // The ones that were passed over were never uploaded.
    assert!(seen
        .iter()
        .all(|id| *id == Some(WallpaperId::Aurora) || *id == Some(WallpaperId::GlassFacade)));
}
