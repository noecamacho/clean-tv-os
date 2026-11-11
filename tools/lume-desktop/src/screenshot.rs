//! Off-screen rendering of screens to RGBA pixels or PNG files.

use crate::gl::HeadlessGl;
use crate::png_out;
use lume_core::app::{Key, LumeApp, Painter};
use lume_core::display::Display;
use lume_core::render::{Backend, DisplayList, Frame, Gles2Backend, Rect};
use lume_core::text::TextEngine;
use lume_core::theme::{vars, Color, THEME};
use std::path::Path;

/// What a screen needs to draw itself.
pub struct Scene<'a> {
    pub display: &'a Display,
    pub text: &'a mut TextEngine,
    pub list: &'a mut DisplayList,
}

/// An off-screen surface with a renderer, kept across frames.
pub struct Session {
    // Declaration order matters: the backend must be dropped before the GL context.
    backend: Gles2Backend,
    _gl: HeadlessGl,
    display: Display,
    text: TextEngine,
    list: DisplayList,
}

impl Session {
    pub fn new(width: u32, height: u32) -> Result<Session, String> {
        let gl = HeadlessGl::new(width, height)?;
        let backend = Gles2Backend::new(gl.glow())?;
        let display = Display::new(width, height);
        // `LUME_FALLBACK_FONT`: a font file for the letters the embedded font lacks (Cyrillic), as the
        // fonts of the device give them on the TV.
        let text = match std::env::var_os("LUME_FALLBACK_FONT") {
            Some(path) => TextEngine::with_fallback(
                display.scale,
                Box::new(lume_core::text::SystemFonts::new(vec![
                    lume_core::text::FontCandidate {
                        path: path.into(),
                        index: 0,
                    },
                ])),
            ),
            None => TextEngine::new(display.scale),
        };
        Ok(Session {
            backend,
            _gl: gl,
            display,
            text,
            list: DisplayList::new(),
        })
    }

    /// Builds one frame. The glyph atlas may be rebuilt while drawing, which asks for the
    /// frame to be built again; this repeats until it settles.
    pub fn build(&mut self, mut draw: impl FnMut(&mut Scene)) {
        for _ in 0..4 {
            self.list.clear();
            self.text.begin_frame();
            draw(&mut Scene {
                display: &self.display,
                text: &mut self.text,
                list: &mut self.list,
            });
            if !self.text.end_frame() {
                break;
            }
        }
    }

    /// Renders the last built frame and reads it back as RGBA rows, top row first.
    pub fn render(&mut self) -> Vec<u8> {
        self.backend.sync_text_atlas(&mut self.text);
        let clear = match THEME.app.base.background {
            Some(lume_core::theme::Paint::Solid(color)) => color,
            _ => Color::rgb(0, 0, 0),
        };
        self.backend
            .render(&self.display.viewport(), clear, &Frame::build(&self.list));
        self.backend
            .read_pixels(self.display.width, self.display.height)
    }

    /// Turns the dither of the wallpaper on or off, to compare the two.
    pub fn set_wallpaper_dither(&mut self, enabled: bool) {
        self.backend.set_wallpaper_dither(enabled);
    }

    /// Puts the demo's synthetic banners and icons in the app's cells of the image texture.
    /// With `LUME_NO_SYNTHETIC_ICONS` set it puts nothing there, so the tiles show their color and letter
    /// (the look of the README pictures).
    pub fn load_icons(&mut self, app: &mut LumeApp) {
        if std::env::var_os("LUME_NO_SYNTHETIC_ICONS").is_some() {
            return;
        }
        let layout = lume_core::app_icons::AtlasLayout::for_scale(self.display.scale);
        app.icons.set_layout(layout);
        self.backend.ensure_image(layout.width, layout.height);
        for (region, pixels) in crate::demo::synthetic_icons(app) {
            self.backend.update_image(region, &pixels);
        }
    }

    pub fn finish(self) {
        self.backend.destroy();
    }
}

/// Renders `draw` on a `width` x `height` surface and returns RGBA rows, top row first.
pub fn render(width: u32, height: u32, draw: impl FnMut(&mut Scene)) -> Result<Vec<u8>, String> {
    let mut session = Session::new(width, height)?;
    session.build(draw);
    let pixels = session.render();
    session.finish();
    Ok(pixels)
}

/// Renders `draw` and writes a PNG.
pub fn save(
    path: &Path,
    width: u32,
    height: u32,
    draw: impl FnMut(&mut Scene),
) -> Result<(), String> {
    let pixels = render(width, height, draw)?;
    png_out::write_rgba(path, width, height, &pixels)
}

/// The "empty" screen: only the theme's background.
pub fn draw_empty(_scene: &mut Scene) {}

/// The launcher after pressing `keys` (one every 100 ms), drawn once everything has settled. With
/// `splash`, lume then leaves the screen and comes back, and the picture is the frame `splash` ms
/// after the first splash frame.
pub fn render_app(
    width: u32,
    height: u32,
    app: &mut LumeApp,
    keys: &[Key],
    splash: Option<u64>,
) -> Result<Vec<u8>, String> {
    let mut session = Session::new(width, height)?;
    session.load_icons(app);
    let mut now = 0u64;
    let frame = |session: &mut Session, app: &mut LumeApp, now: u64| {
        session.build(|scene| {
            let mut painter = Painter {
                list: scene.list,
                text: scene.text,
                display: *scene.display,
            };
            crate::demo::feed_stats(app, now);
            app.draw(&mut painter, now);
        });
    };
    frame(&mut session, app, now);
    for key in keys {
        now += 100;
        app.handle_key(*key, now);
        frame(&mut session, app, now);
        if std::env::var_os("LUME_TRACE").is_some() {
            eprintln!("{key:?} -> focus {:?}", app.focused());
        }
    }
    // Let every animation finish before the picture that is kept.
    now += 2000;
    frame(&mut session, app, now);
    if let Some(after) = splash {
        app.went_to_background();
        app.came_to_foreground();
        let first = now + 100;
        // Frames as the loop would draw them, so the splash sees its deadline and the reveal runs.
        let mut at = first;
        while at < first + after {
            frame(&mut session, app, at);
            at += 16;
        }
        now = first + after;
    }
    // A glyph first drawn in a pass that rebuilds the atlas can still be missing from that pass: one more
    // frame, with everything already in the atlas, is the one that is kept.
    frame(&mut session, app, now);
    let pixels = session.render();
    session.finish();
    Ok(pixels)
}

/// The logo lockup on the theme background: mark, wordmark and tagline, as in the design file.
pub fn draw_logo(scene: &mut Scene) {
    use lume_core::logo;
    use lume_core::text::{Align, TextStyle};

    let (mark_size, center_x) = (168.0, 280.0);
    logo::draw_mark(
        scene.list,
        Rect::new(center_x - mark_size / 2.0, 135.5, mark_size, mark_size),
        vars::ACCENT,
        vars::BG,
    );
    let wordmark = TextStyle {
        size: 68.0,
        weight: 800,
        letter_spacing: -1.5,
    };
    scene.text.draw_line(
        scene.list,
        "lume",
        Rect::new(center_x - 150.0, 330.0, 300.0, 70.0),
        Align::Center,
        &wordmark,
        vars::TEXT,
    );
    let tagline = TextStyle::new(20.0, 500);
    scene.text.draw_line(
        scene.list,
        "seu início na TV",
        Rect::new(center_x - 150.0, 396.0, 300.0, 32.0),
        Align::Center,
        &tagline,
        vars::TEXT_MUTED,
    );
}

/// The TV banner: the logo mark and the name side by side on the theme background.
/// Drawn on the 1280x720 canvas; render it at 16:9 sizes such as 320x180 or 640x360.
pub fn draw_banner(scene: &mut Scene) {
    use lume_core::logo;
    use lume_core::text::{Align, TextStyle};

    let mark = 232.0;
    let wordmark = TextStyle {
        size: 190.0,
        weight: 800,
        letter_spacing: -4.0,
    };
    let text_width = scene.text.measure("lume", &wordmark);
    let gap = 56.0;
    let left = (1280.0 - (mark + gap + text_width)) / 2.0;
    let center_y = 360.0;
    logo::draw_mark(
        scene.list,
        Rect::new(left, center_y - mark / 2.0, mark, mark),
        vars::ACCENT,
        vars::BG,
    );
    scene.text.draw_line(
        scene.list,
        "lume",
        Rect::new(left + mark + gap, center_y - 90.0, text_width + 20.0, 180.0),
        Align::Start,
        &wordmark,
        vars::TEXT,
    );
}
