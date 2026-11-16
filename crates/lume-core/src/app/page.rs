//! The header of the secondary screens: breadcrumb path and title.

use super::painter::Painter;
use crate::theme::{StateFlags, THEME};

pub const LEFT: f32 = 64.0;
pub const WIDTH: f32 = 1152.0;
const TITLE_BASELINE: f32 = 68.0;
const SUBTITLE_BASELINE: f32 = 104.0;
/// Where the content of a secondary screen starts: 58 below the subtitle's baseline.
pub const CONTENT_TOP: f32 = 212.0;

/// Draws the path (`Settings › Home screen`, none on the Settings screen itself) and title.
/// `scroll_y` moves everything up when the page scrolls.
pub fn draw_header(
    p: &mut Painter,
    crumbs: &[&str],
    title: &str,
    subtitle: Option<&str>,
    scroll_y: f32,
) {
    // As on tvOS: the title alone, centered; the way back is the Back button, not a trail.
    let _ = crumbs;
    let title_style = p.style(&THEME.page_title, StateFlags::NONE);
    let text_style = Painter::text_style(&title_style);
    let shown = p.text.ellipsize(title, &text_style, 900.0).into_owned();
    let width = p.text.measure(&shown, &text_style);
    p.text.draw(
        p.list,
        &shown,
        (1280.0 - width) / 2.0,
        TITLE_BASELINE - scroll_y,
        &text_style,
        title_style.color,
    );
    if let Some(subtitle) = subtitle {
        let style = p.style(&THEME.page_subtitle, StateFlags::NONE);
        let text_style = Painter::text_style(&style);
        let shown = p.text.ellipsize(subtitle, &text_style, WIDTH).into_owned();
        let width = p.text.measure(&shown, &text_style);
        p.text.draw(
            p.list,
            &shown,
            (1280.0 - width) / 2.0,
            SUBTITLE_BASELINE - scroll_y,
            &text_style,
            style.color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_is_on_the_ruler() {
        super::super::assert_on_ruler(&[
            ("LEFT", LEFT),
            ("WIDTH", WIDTH),
            ("TITLE_BASELINE", TITLE_BASELINE),
            ("SUBTITLE_BASELINE", SUBTITLE_BASELINE),
            ("CONTENT_TOP", CONTENT_TOP),
        ]);
    }
}
