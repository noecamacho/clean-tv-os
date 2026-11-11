//! Measures the text contrast on the rendered frames (real OpenGL ES 2.0 backend, Mesa software
//! rendering on CI). Tests are skipped when no EGL implementation is available.

use lume_core::app::{Key, LumeApp};
use lume_core::catalog::Catalog;
use lume_core::clock::WallClock;
use lume_core::home::config::HomeConfig;
use lume_core::i18n::Translator;
use lume_core::theme::{vars, Color};
use lume_core::usage::UsageHistory;
use lume_desktop::screenshot;

const W: u32 = 1280;
const H: u32 = 720;

/// A launcher with no apps: the only tile is "Settings", so most of the screen is wallpaper.
fn empty_app() -> LumeApp {
    let catalog = Catalog::from_discovered(&[], "app.lume.tv");
    let config = HomeConfig::first_use(&catalog);
    let clock = WallClock {
        unix_secs: lume_desktop::demo::MOCKUP_TIME,
        utc_offset_secs: 0,
        is_24h: true,
    };
    LumeApp::new(
        catalog,
        UsageHistory::new(),
        config,
        Translator::new(&["en"]),
        clock,
    )
}

fn luminance([r, g, b]: [u8; 3]) -> f32 {
    let linear = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(r) + 0.7152 * linear(g) + 0.0722 * linear(b)
}

fn contrast(text: Color, background: f32) -> f32 {
    let text = luminance([text.r, text.g, text.b]);
    (text.max(background) + 0.05) / (text.min(background) + 0.05)
}

/// How bright the wallpaper is in the rectangle, leaving out the `covered` rectangles: the level
/// 99.5% of its pixels stay under. A point of a star is not a background a text sits on, so the
/// few star pixels do not count; everything else does.
fn brightest(pixels: &[u8], area: (u32, u32, u32, u32), covered: &[(u32, u32, u32, u32)]) -> f32 {
    let mut levels = Vec::new();
    for y in area.1..area.3 {
        for x in area.0..area.2 {
            if covered
                .iter()
                .any(|c| x >= c.0 && x < c.2 && y >= c.1 && y < c.3)
            {
                continue;
            }
            let i = ((y * W + x) * 4) as usize;
            levels.push(luminance([pixels[i], pixels[i + 1], pixels[i + 2]]));
        }
    }
    levels.sort_by(f32::total_cmp);
    levels
        .get(((levels.len() as f32 * 0.995) as usize).min(levels.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0.0)
}

fn render(keys: &[Key]) -> Option<Vec<u8>> {
    let mut app = empty_app();
    match screenshot::render_app(W, H, &mut app, keys, None) {
        Ok(pixels) => Some(pixels),
        Err(e) if e.contains("EGL") || e.contains("GL context") => {
            eprintln!("skipped: {e}");
            None
        }
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn the_apps_screen_text_has_4_5_to_1_over_the_brightest_wallpaper_pixel() {
    let Some(pixels) = render(&[Key::Up, Key::Right, Key::Ok]) else {
        return;
    };
    // Everything below the top bar except the title, the only tile and its name (top left): there
    // is nothing but the wallpaper and the dimming layer, so the brightest pixel is the worst case.
    let background = brightest(&pixels, (0, 88, W, H), &[(0, 88, 300, 300)]);
    let ratio = contrast(vars::TEXT_MUTED, background);
    assert!(
        ratio >= 4.5,
        "muted text over the brightest pixel: {ratio:.2}:1 (luminance {background:.4})"
    );
}

#[test]
fn the_settings_and_about_screens_are_dimmed_too() {
    let Some(settings) = render(&[Key::Up, Key::Right, Key::Down, Key::Ok]) else {
        return;
    };
    let about = render(&[Key::Up, Key::Right, Key::Down, Key::Ok, Key::Down, Key::Ok]).unwrap();
    for (name, pixels) in [("settings", settings), ("about", about)] {
        // The right half below the header has no text on these screens, except the statistics
        // card (which has its own surface) on the settings: look below it. The version line at the
        // bottom right is text too (its length depends on the build), so it is left out.
        let background = brightest(&pixels, (800, 500, W, H), &[(800, 620, W, H)]);
        let ratio = contrast(vars::TEXT_MUTED, background);
        assert!(
            ratio >= 4.5,
            "{name}: {ratio:.2}:1 (luminance {background:.4})"
        );
    }
}

#[test]
fn the_home_top_bar_keeps_4_5_to_1_without_the_dimming_layer() {
    let Some(pixels) = render(&[]) else { return };
    // The strip just under the tab labels (no text, and brighter than the rows of the text itself
    // because the top of the wallpaper gets darker upward): the inactive tab is the dimmest text.
    let tabs = brightest(&pixels, (220, 58, 390, 63), &[]);
    let ratio = contrast(vars::TEXT_MUTED, tabs);
    assert!(
        ratio >= 4.5,
        "inactive tab: {ratio:.2}:1 (luminance {tabs:.4})"
    );
    // Home is not dimmed: below the favorites the wallpaper is brighter than the dimming allows.
    let home_body = brightest(&pixels, (0, 400, W, H), &[]);
    assert!(
        home_body > 0.1,
        "the wallpaper shows undimmed: {home_body:.4}"
    );
}
