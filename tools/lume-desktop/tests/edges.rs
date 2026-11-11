//! The edges of the shapes that carry an image (design-system "Uma única borda suavizada por
//! forma", "Contorno de foco sem fresta" and "Largura da suavização das bordas"), measured on frames
//! of the real OpenGL ES 2.0 backend with the demo's synthetic pictures: an opaque white icon for
//! each favorite and a solid banner for each tile. Tests are skipped when no EGL implementation is
//! available.

use lume_core::app::{Key, LumeApp};
use lume_desktop::demo::{self, Setup, SYNTHETIC_BANNER};
use lume_desktop::screenshot;

const SIZES: [(u32, u32); 2] = [(1280, 720), (1920, 1080)];
const WHITE: [f32; 3] = [255.0, 255.0, 255.0];
const BLUE: [f32; 3] = [77.0, 179.0, 255.0]; // --primary
const BANNER: [f32; 3] = [
    ((SYNTHETIC_BANNER >> 16) & 0xFF) as f32,
    ((SYNTHETIC_BANNER >> 8) & 0xFF) as f32,
    (SYNTHETIC_BANNER & 0xFF) as f32,
];

struct Shot {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

impl Shot {
    fn at(&self, x: i32, y: i32) -> [f32; 3] {
        let x = x.clamp(0, self.width as i32 - 1) as u32;
        let y = y.clamp(0, self.height as i32 - 1) as u32;
        let i = ((y * self.width + x) * 4) as usize;
        [
            f32::from(self.pixels[i]),
            f32::from(self.pixels[i + 1]),
            f32::from(self.pixels[i + 2]),
        ]
    }
}

/// The demo launcher with three favorites, round when `round`, after `keys`.
fn render(width: u32, height: u32, keys: &[Key], round: bool) -> Option<Shot> {
    let mut app: LumeApp = demo::app_with(&Setup {
        locale: Some("en".into()),
        favorites: Some(3),
        ..Setup::default()
    });
    app.config.round_favorites = round;
    match screenshot::render_app(width, height, &mut app, keys, None) {
        Ok(pixels) => Some(Shot {
            pixels,
            width,
            height,
        }),
        Err(e) if e.contains("EGL") || e.contains("GL context") => {
            eprintln!("skipped: {e}");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Where `c` falls between `from` (0.0) and `to` (1.0), and how far it is from that line.
fn on_segment(c: [f32; 3], from: [f32; 3], to: [f32; 3]) -> (f32, f32) {
    let axis = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let length2 = axis.iter().map(|v| v * v).sum::<f32>().max(1e-6);
    let t = ((c[0] - from[0]) * axis[0] + (c[1] - from[1]) * axis[1] + (c[2] - from[2]) * axis[2])
        / length2;
    let t = t.clamp(0.0, 1.0);
    let point = [
        from[0] + axis[0] * t,
        from[1] + axis[1] * t,
        from[2] + axis[2] * t,
    ];
    (t, distance(c, point))
}

/// The favorites' icons: bounding boxes of the opaque white blobs in the lower part of the screen.
fn favorites(shot: &Shot) -> Vec<(i32, i32, i32, i32)> {
    let (w, h) = (shot.width as i32, shot.height as i32);
    let top = h * 6 / 10;
    let white = |x: i32, y: i32| shot.at(x, y).iter().all(|&c| c >= 250.0);
    let mut seen = vec![false; (w * h) as usize];
    let mut boxes = Vec::new();
    for y in top..h {
        for x in 0..w {
            if seen[(y * w + x) as usize] || !white(x, y) {
                continue;
            }
            let (mut x0, mut y0, mut x1, mut y1, mut count) = (x, y, x, y, 0);
            let mut stack = vec![(x, y)];
            seen[(y * w + x) as usize] = true;
            while let Some((px, py)) = stack.pop() {
                count += 1;
                (x0, y0, x1, y1) = (x0.min(px), y0.min(py), x1.max(px), y1.max(py));
                for (nx, ny) in [(px + 1, py), (px - 1, py), (px, py + 1), (px, py - 1)] {
                    if nx >= 0 && ny >= top && nx < w && ny < h {
                        let i = (ny * w + nx) as usize;
                        if !seen[i] && white(nx, ny) {
                            seen[i] = true;
                            stack.push((nx, ny));
                        }
                    }
                }
            }
            if count > 400 {
                boxes.push((x0, y0, x1, y1));
            }
        }
    }
    boxes
}

/// The pixels from inside a favorite's icon to past its edge, in direction `(dx, dy)` (unit steps
/// of whole pixels), with the background a few pixels outside.
fn favorite_edge(
    shot: &Shot,
    (x0, y0, x1, y1): (i32, i32, i32, i32),
    (dx, dy): (i32, i32),
) -> (Vec<[f32; 3]>, [f32; 3]) {
    let (cx, cy) = ((x0 + x1) / 2, (y0 + y1) / 2);
    // How far one step goes: 1 along an axis, the square root of 2 along a diagonal.
    let step = ((dx * dx + dy * dy) as f32).sqrt();
    let reach = (x1 - x0) as f32 / 2.0;
    let steps = |distance: f32| (distance / step).round() as i32;
    let line: Vec<_> = (steps(reach * 0.7)..=steps(reach + 3.0))
        .map(|i| shot.at(cx + dx * i, cy + dy * i))
        .collect();
    let past = steps(reach + 4.5);
    (line, shot.at(cx + dx * past, cy + dy * past))
}

const DIRECTIONS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

#[test]
fn a_round_favorite_has_no_ring_of_the_app_color_around_its_icon() {
    for (width, height) in SIZES {
        let Some(shot) = render(width, height, &[], true) else {
            return;
        };
        let icons = favorites(&shot);
        assert_eq!(icons.len(), 3, "{width}x{height}: three favorites");
        for icon in icons {
            for direction in DIRECTIONS {
                let (line, outside) = favorite_edge(&shot, icon, direction);
                for pixel in line {
                    let (_, off) = on_segment(pixel, WHITE, outside);
                    assert!(
                        off <= 12.0,
                        "{width}x{height}, favorite {icon:?}, direction {direction:?}: {pixel:?} is \
                         not a mix of the icon and the background {outside:?} ({off:.1} away)"
                    );
                }
            }
        }
    }
}

#[test]
fn the_edge_of_a_round_favorite_takes_one_to_two_pixels() {
    for (width, height) in SIZES {
        let Some(shot) = render(width, height, &[], true) else {
            return;
        };
        // The first favorite has the focus and is at the peak of the Magnification.
        for icon in favorites(&shot) {
            for direction in &DIRECTIONS[..4] {
                let (line, outside) = favorite_edge(&shot, icon, *direction);
                let partial = line
                    .iter()
                    .filter(|&&pixel| {
                        let (t, _) = on_segment(pixel, WHITE, outside);
                        t > 0.04 && t < 0.96
                    })
                    .count();
                assert!(
                    (1..=2).contains(&partial),
                    "{width}x{height}, favorite {icon:?}, direction {direction:?}: the edge takes \
                     {partial} pixels"
                );
            }
        }
    }
}

/// The focus outline of the focused tile, as the box of the pixels of `--primary` in the first
/// row of tiles.
fn focus_ring(shot: &Shot) -> (i32, i32, i32, i32) {
    let (w, h) = (shot.width as i32, shot.height as i32);
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in h * 15 / 100..h * 38 / 100 {
        for x in 0..w {
            if distance(shot.at(x, y), BLUE) < 12.0 {
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
            }
        }
    }
    assert!(x1 > x0 && y1 > y0, "no focus outline found");
    (x0, y0, x1, y1)
}

fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

#[test]
fn no_dark_line_between_the_focus_outline_and_the_banner() {
    // Apps tab, focus on the second tile of the first row.
    let keys = [Key::Up, Key::Right, Key::Down, Key::Right];
    for (width, height) in SIZES {
        let Some(shot) = render(width, height, &keys, false) else {
            return;
        };
        let (x0, y0, x1, y1) = focus_ring(&shot);
        let (cx, cy) = ((x0 + x1) / 2, (y0 + y1) / 2);
        // Into the banner from the left, the top and the top left corner.
        let walks = [
            ("left", (x0 - 2, cy), (1, 0)),
            ("top", (cx, y0 - 2), (0, 1)),
            ("corner", (x0 - 2, y0 - 2), (1, 1)),
        ];
        for (name, (sx, sy), (dx, dy)) in walks {
            let line: Vec<_> = (0..(cy - y0))
                .map(|i| shot.at(sx + dx * i, sy + dy * i))
                .collect();
            let first_blue = line
                .iter()
                .position(|&c| distance(c, BLUE) < 30.0)
                .unwrap_or_else(|| panic!("{width}x{height} {name}: no outline"));
            let first_banner = first_blue
                + line[first_blue..]
                    .iter()
                    .position(|&c| distance(c, BANNER) < 30.0)
                    .unwrap_or_else(|| panic!("{width}x{height} {name}: no banner"));
            // Between the outline and the banner every pixel is a mix of the two, never darker
            // than both (the wallpaper or the tile color showing through).
            let floor = luma(BLUE).min(luma(line[first_banner])) - 2.0;
            for (i, &pixel) in line.iter().enumerate().take(first_banner).skip(first_blue) {
                assert!(
                    luma(pixel) >= floor,
                    "{width}x{height} {name}: pixel {i} {pixel:?} between the outline and the \
                     banner is darker than both (luma {:.1} < {floor:.1})",
                    luma(pixel)
                );
            }
        }
    }
}
