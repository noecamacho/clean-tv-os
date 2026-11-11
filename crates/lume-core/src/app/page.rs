//! The header of the secondary screens: breadcrumb path and title.

use super::painter::Painter;
use crate::theme::{StateFlags, THEME};

pub const LEFT: f32 = 64.0;
pub const WIDTH: f32 = 1152.0;
const BREADCRUMB_BASELINE: f32 = 52.0;
const TITLE_BASELINE: f32 = 116.0;
const SUBTITLE_BASELINE: f32 = 152.0;
/// Where the content of a secondary screen starts: 58 below the subtitle's baseline.
pub const CONTENT_TOP: f32 = 212.0;
/// Space on each side of the "›" between two steps of the breadcrumb.
const SEPARATOR_GAP: f32 = 8.0;

/// Draws the path (`Settings › Home screen`, none on the Settings screen itself) and title.
/// `scroll_y` moves everything up when the page scrolls.
pub fn draw_header(
    p: &mut Painter,
    crumbs: &[&str],
    title: &str,
    subtitle: Option<&str>,
    scroll_y: f32,
) {
    let muted = p.style(&THEME.breadcrumb, StateFlags::NONE);
    let current = p.style(&THEME.breadcrumb, StateFlags::ACTIVE);
    let mut x = LEFT;
    for (index, crumb) in crumbs.iter().enumerate() {
        let last = index + 1 == crumbs.len();
        let style = if last { &current } else { &muted };
        let text_style = Painter::text_style(style);
        p.text.draw(
            p.list,
            crumb,
            x,
            BREADCRUMB_BASELINE - scroll_y,
            &text_style,
            style.color,
        );
        x += p.text.measure(crumb, &text_style);
        if !last {
            let separator = "›";
            x += SEPARATOR_GAP;
            p.text.draw(
                p.list,
                separator,
                x,
                BREADCRUMB_BASELINE - scroll_y,
                &text_style,
                muted.color,
            );
            x += p.text.measure(separator, &text_style) + SEPARATOR_GAP;
        }
    }
    let title_style = p.style(&THEME.page_title, StateFlags::NONE);
    let shown = p
        .text
        .ellipsize(title, &Painter::text_style(&title_style), 900.0)
        .into_owned();
    p.text.draw(
        p.list,
        &shown,
        LEFT,
        TITLE_BASELINE - scroll_y,
        &Painter::text_style(&title_style),
        title_style.color,
    );
    if let Some(subtitle) = subtitle {
        let style = p.style(&THEME.page_subtitle, StateFlags::NONE);
        let shown = p
            .text
            .ellipsize(subtitle, &Painter::text_style(&style), WIDTH)
            .into_owned();
        p.text.draw(
            p.list,
            &shown,
            LEFT,
            SUBTITLE_BASELINE - scroll_y,
            &Painter::text_style(&style),
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
            ("BREADCRUMB_BASELINE", BREADCRUMB_BASELINE),
            ("TITLE_BASELINE", TITLE_BASELINE),
            ("SUBTITLE_BASELINE", SUBTITLE_BASELINE),
            ("CONTENT_TOP", CONTENT_TOP),
            ("SEPARATOR_GAP", SEPARATOR_GAP),
        ]);
    }
}
