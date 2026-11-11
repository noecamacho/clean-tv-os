//! The lume settings screen and the About screen.

use super::page::{self, LEFT, WIDTH};
use super::painter::Painter;
use super::{ids, Effect, LumeApp, PendingFocus, Screen, Tab};
use crate::device_info::SECTIONS;
use crate::i18n::{Arg, StringId};
use crate::icons;
use crate::licenses;
use crate::render::Rect;
use crate::stats;
use crate::text::Align;
use crate::theme::Paint;
use crate::theme::{vars, ComputedStyle, StateFlags, THEME};
use crate::ui::focus::{Focusable, Row};

/// Where the source code lives, shown on the About screen.
pub const SOURCE_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// The settings screen is split in two equal columns with a 48 px gutter: items and statistics.
const COLUMN_WIDTH: f32 = (WIDTH - 48.0) / 2.0;
const ITEM_WIDTH: f32 = COLUMN_WIDTH;
const ITEMS_TOP: f32 = page::CONTENT_TOP;
const ITEM_PITCH: f32 = 88.0;
const ABOUT_TEXT_TOP: f32 = 192.0;
const ABOUT_VIEW_TOP: f32 = 152.0;
const ABOUT_STEP: f32 = 96.0;
/// The TV information sheet: labels in a column at the left, values in a second one.
const TV_VIEW_TOP: f32 = 152.0;
const TV_TEXT_TOP: f32 = 192.0;
const TV_STEP: f32 = 96.0;
const TV_HEADING_HEIGHT: f32 = 32.0;
const TV_ROW_HEIGHT: f32 = 40.0;
const TV_SECTION_GAP: f32 = 32.0;
const TV_VALUE_X: f32 = LEFT + 384.0;
const PANEL_X: f32 = LEFT + COLUMN_WIDTH + 48.0;
const PANEL_WIDTH: f32 = COLUMN_WIDTH;
const PANEL_HEIGHT: f32 = 312.0;

pub(super) const ITEM_HOME_SCREEN: u32 = ids::SETTINGS_BASE;
pub(super) const ITEM_KEY_MAPPING: u32 = ids::SETTINGS_BASE + 1;
pub(super) const ITEM_TV_INFO: u32 = ids::SETTINGS_BASE + 2;
pub(super) const ITEM_ABOUT: u32 = ids::SETTINGS_BASE + 3;

/// The line height of running text on the ruler (see the `design-system` spec): 32 for captions,
/// 40 for body and labels, 44 for the primary size and above.
fn line_height(font_size: f32) -> f32 {
    if font_size <= 16.0 {
        32.0
    } else if font_size <= 20.0 {
        40.0
    } else {
        44.0
    }
}

impl LumeApp {
    pub(super) fn open_settings(&mut self) {
        self.screen = Screen::Settings;
        self.stats = None;
        self.cpu_use = None;
        self.stats_at = None;
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::FirstContent);
        // The settings screen has no top bar: its first row is the first item.
        self.focus.set_rows(vec![]);
    }

    pub(super) fn draw_settings(&mut self, p: &mut Painter) {
        let t = self.translator;
        let labels = [
            (
                ITEM_HOME_SCREEN,
                t.get(StringId::SettingsHomeScreen).into_owned(),
            ),
            (
                ITEM_KEY_MAPPING,
                t.get(StringId::SettingsKeyMapping).into_owned(),
            ),
            (ITEM_TV_INFO, t.get(StringId::SettingsTvInfo).into_owned()),
            (ITEM_ABOUT, t.get(StringId::SettingsAbout).into_owned()),
        ];
        let rects: Vec<Rect> = (0..labels.len())
            .map(|i| Rect::new(LEFT, ITEMS_TOP + i as f32 * ITEM_PITCH, ITEM_WIDTH, 72.0))
            .collect();
        let rows = labels
            .iter()
            .zip(&rects)
            .map(|((id, _), rect)| Row::new(vec![Focusable::new(*id, *rect)]))
            .collect();
        self.update_focus(rows);

        page::draw_header(
            p,
            &[&t.get(StringId::SettingsTitle)],
            &t.get(StringId::SettingsTitle),
            None,
            0.0,
        );
        for ((id, label), rect) in labels.iter().zip(&rects) {
            let progress = self.focus_progress(*id);
            let style = p.focus_style(&THEME.settings_item, progress);
            p.draw_box(*rect, &style);
            let inner = Rect::new(
                rect.x + style.padding[3],
                rect.y,
                rect.w - style.padding[3] - 70.0,
                rect.h,
            );
            p.text(label, inner, Align::Start, &style);
            let chevron = Rect::new(rect.right() - 28.0 - 28.0, rect.y, 28.0, rect.h);
            p.icon(icons::CHEVRON_RIGHT, chevron, 24.0, style.color);
        }
        self.draw_stats(p);

        // The installed version, at the bottom right.
        let version = t.format(
            StringId::AboutVersion,
            &[Arg::Str(&lume_version::display())],
        );
        let style = p.style(&THEME.breadcrumb, StateFlags::NONE);
        let bottom = p.display.safe_area().bottom();
        let line = Rect::new(LEFT, bottom - 40.0, WIDTH, 24.0);
        p.text(&version, line, Align::End, &style);
    }

    /// The TV statistics card, right of the items: no focus, no animation.
    fn draw_stats(&self, p: &mut Painter) {
        let t = self.translator;
        let comma = t.language().starts_with("pt");
        let panel = Rect::new(PANEL_X, ITEMS_TOP, PANEL_WIDTH, PANEL_HEIGHT);
        let style = p.style(&THEME.panel, StateFlags::NONE);
        p.draw_box(panel, &style);
        let inner = Rect::new(panel.x + 32.0, panel.y + 24.0, panel.w - 64.0, 20.0);
        let heading = p.style(&THEME.card_heading, StateFlags::NONE);
        p.text(&t.get(StringId::StatsTitle), inner, Align::Start, &heading);

        let stats = self.stats.as_ref();
        let rows = [
            (
                StringId::StatsMemory,
                stats.map(|s| s.memory_used()),
                stats.map(|s| {
                    format!(
                        "{} / {}",
                        stats::format_size(s.memory_used_bytes(), comma),
                        stats::format_size(s.memory_total_bytes(), comma)
                    )
                }),
            ),
            (
                StringId::StatsDisk,
                stats.map(|s| s.disk_used()),
                stats.map(|s| {
                    format!(
                        "{} / {}",
                        stats::format_size(s.disk_used_bytes(), comma),
                        stats::format_size(s.disk_total, comma)
                    )
                }),
            ),
            (
                StringId::StatsCpu,
                self.cpu_use,
                match stats.map(|s| s.cpu) {
                    Some(stats::CpuReading::Freq { current, max }) if max > 0 => Some(format!(
                        "{} / {}",
                        stats::format_freq(current, comma),
                        stats::format_freq(max, comma)
                    )),
                    _ => self.cpu_use.map(|u| format!("{:.0}%", u * 100.0)),
                },
            ),
        ];
        let label = p.style(&THEME.breadcrumb, StateFlags::NONE);
        let mut value_style = label;
        value_style.color = vars::TEXT;
        value_style.font_weight = 700;
        for (index, (id, fraction, value)) in rows.into_iter().enumerate() {
            let top = panel.y + 72.0 + index as f32 * 80.0;
            let line = Rect::new(inner.x, top, inner.w, 24.0);
            p.text(&t.get(id), line, Align::Start, &label);
            // A dash stands for what the system would not tell.
            let value = value.unwrap_or_else(|| "—".to_string());
            p.text(&value, line, Align::End, &value_style);
            let bar = Rect::new(inner.x, top + 36.0, inner.w, 8.0);
            p.list.fill(bar, 4.0, Paint::Solid(vars::CONTROL));
            if let Some(fraction) = fraction {
                let filled = Rect::new(bar.x, bar.y, (bar.w * fraction).max(8.0), bar.h);
                p.list.fill(filled, 4.0, Paint::Solid(vars::PRIMARY));
            }
        }
    }

    pub(super) fn activate_settings(&mut self, id: u32) -> Vec<Effect> {
        match id {
            ITEM_HOME_SCREEN => {
                self.open_editor(false);
                vec![]
            }
            ITEM_KEY_MAPPING => {
                self.open_keymap();
                vec![]
            }
            ITEM_TV_INFO => {
                self.open_tv_info();
                vec![]
            }
            ITEM_ABOUT => {
                self.screen = Screen::About;
                self.about_scroll = 0.0;
                self.focus.set_rows(vec![]);
                vec![]
            }
            _ => vec![],
        }
    }

    pub(super) fn back_from_settings(&mut self) {
        self.screen = Screen::Main(Tab::Apps);
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::Id(ids::GRID_SETTINGS));
    }

    pub(super) fn open_tv_info(&mut self) {
        self.screen = Screen::TvInfo;
        // The sheet is read again each time it opens (the address and the free space change).
        self.device_info = None;
        self.tv_max_scroll = 0.0;
        self.scroll_to_top();
        self.focus.set_rows(vec![]);
    }

    pub(super) fn back_from_tv_info(&mut self) {
        self.screen = Screen::Settings;
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::Id(ITEM_TV_INFO));
    }

    /// Scrolls the TV information with the up and down keys, smoothly and within the sheet.
    pub(super) fn scroll_tv_info(&mut self, delta: f32) {
        let target = (self.scroll_y + delta * TV_STEP).clamp(0.0, self.tv_max_scroll);
        self.scroll_to(target);
    }

    pub(super) fn draw_tv_info(&mut self, p: &mut Painter) {
        let t = self.translator;
        self.update_focus(vec![]);
        let visible = p.display.visible_rect();
        let view = (visible.bottom() - TV_VIEW_TOP).max(1.0);

        // The sheet's height decides how far it scrolls.
        let total: f32 = SECTIONS
            .iter()
            .map(|s| TV_HEADING_HEIGHT + s.lines.len() as f32 * TV_ROW_HEIGHT + TV_SECTION_GAP)
            .sum();
        self.tv_max_scroll = (TV_TEXT_TOP - TV_VIEW_TOP + total - view).max(0.0);
        if self.scroll_y > self.tv_max_scroll {
            self.scroll_to(self.tv_max_scroll);
        }
        let scroll = self.current_scroll_y();

        // Nothing is drawn until the platform has read the sheet (a turn of the loop).
        if let Some(info) = self.device_info.as_ref() {
            let heading = p.style(&THEME.card_heading, StateFlags::NONE);
            let label = p.style(&THEME.breadcrumb, StateFlags::NONE);
            let mut value_style = label;
            value_style.color = vars::TEXT;
            value_style.font_weight = 700;
            p.list.push_clip(Rect::new(
                visible.x,
                TV_VIEW_TOP,
                visible.w,
                visible.bottom() - TV_VIEW_TOP,
            ));
            let mut y = TV_TEXT_TOP - scroll;
            let view_bottom = visible.bottom();
            for section in &SECTIONS {
                let rect = Rect::new(LEFT, y, WIDTH, TV_HEADING_HEIGHT);
                if y + TV_HEADING_HEIGHT >= TV_VIEW_TOP && y <= view_bottom {
                    p.text(&t.get(section.title), rect, Align::Start, &heading);
                }
                y += TV_HEADING_HEIGHT;
                for (field, name) in section.lines {
                    let name_rect = Rect::new(LEFT, y, TV_VALUE_X - LEFT - 32.0, TV_ROW_HEIGHT);
                    let value_rect =
                        Rect::new(TV_VALUE_X, y, LEFT + WIDTH - TV_VALUE_X, TV_ROW_HEIGHT);
                    if y + TV_ROW_HEIGHT >= TV_VIEW_TOP && y <= view_bottom {
                        p.text(&t.get(*name), name_rect, Align::Start, &label);
                        let value = info.display(*field, &t);
                        p.text(&value, value_rect, Align::Start, &value_style);
                    }
                    y += TV_ROW_HEIGHT;
                }
                y += TV_SECTION_GAP;
            }
            p.list.pop_clip();
        }

        page::draw_header(
            p,
            &[
                &t.get(StringId::SettingsTitle),
                &t.get(StringId::SettingsTvInfo),
            ],
            &t.get(StringId::TvInfoTitle),
            None,
            0.0,
        );
    }

    /// Scrolls the About text with the up and down keys.
    pub(super) fn scroll_about(&mut self, delta: f32) {
        self.about_scroll = (self.about_scroll + delta * ABOUT_STEP).max(0.0);
    }

    pub(super) fn draw_about(&mut self, p: &mut Painter) {
        let t = self.translator;
        self.update_focus(vec![]);
        let visible = p.display.visible_rect();

        // Build the text, then limit the scrolling to its length.
        let lines = licenses::cached_lines();
        let heading = p.style(&THEME.about_heading, StateFlags::NONE);
        let body = p.style(&THEME.about_body, StateFlags::NONE);
        let license = p.style(&THEME.about_license, StateFlags::NONE);
        let mut blocks: Vec<(std::borrow::Cow<'static, str>, &ComputedStyle)> = vec![
            (
                t.format(
                    StringId::AboutVersion,
                    &[Arg::Str(&lume_version::display())],
                )
                .into(),
                &heading,
            ),
            (t.get(StringId::AboutLicense), &body),
            (
                t.format(StringId::AboutSource, &[Arg::Str(SOURCE_URL)])
                    .into(),
                &body,
            ),
            (std::borrow::Cow::Borrowed(""), &license),
            (t.get(StringId::AboutThirdParty), &heading),
        ];
        blocks.extend(
            lines
                .iter()
                .map(|line| (std::borrow::Cow::Borrowed(line.as_str()), &license)),
        );
        let heights: Vec<f32> = blocks
            .iter()
            .map(|(_, style)| line_height(style.font_size))
            .collect();
        let total: f32 = heights.iter().sum();
        let view = (visible.bottom() - ABOUT_VIEW_TOP).max(1.0);
        let max_scroll = (ABOUT_TEXT_TOP + total + 40.0 - ABOUT_VIEW_TOP - view).max(0.0);
        self.about_scroll = self.about_scroll.min(max_scroll);
        let scroll = self.about_scroll;

        p.list.push_clip(Rect::new(
            visible.x,
            ABOUT_VIEW_TOP,
            visible.w,
            visible.bottom() - ABOUT_VIEW_TOP,
        ));
        let view_bottom = visible.bottom();
        let mut y = ABOUT_TEXT_TOP - scroll;
        for ((text, style), height) in blocks.iter().zip(&heights) {
            let rect = Rect::new(LEFT, y, WIDTH, *height);
            if y + *height >= ABOUT_VIEW_TOP && y <= view_bottom && !text.is_empty() {
                p.text(text, rect, Align::Start, style);
            }
            y += height;
        }
        p.list.pop_clip();

        // The scrolling text is clipped below the header, so the header needs no cover.
        page::draw_header(
            p,
            &[
                &t.get(StringId::SettingsTitle),
                &t.get(StringId::SettingsAbout),
            ],
            &t.get(StringId::AboutTitle),
            None,
            0.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_is_on_the_ruler() {
        super::super::assert_on_ruler(&[
            ("COLUMN_WIDTH", COLUMN_WIDTH),
            ("ITEM_PITCH", ITEM_PITCH),
            ("ABOUT_TEXT_TOP", ABOUT_TEXT_TOP),
            ("ABOUT_VIEW_TOP", ABOUT_VIEW_TOP),
            ("ABOUT_STEP", ABOUT_STEP),
            ("TV_VIEW_TOP", TV_VIEW_TOP),
            ("TV_TEXT_TOP", TV_TEXT_TOP),
            ("TV_STEP", TV_STEP),
            ("TV_HEADING_HEIGHT", TV_HEADING_HEIGHT),
            ("TV_ROW_HEIGHT", TV_ROW_HEIGHT),
            ("TV_SECTION_GAP", TV_SECTION_GAP),
            ("TV_VALUE_X", TV_VALUE_X),
            ("PANEL_X", PANEL_X),
            ("PANEL_HEIGHT", PANEL_HEIGHT),
        ]);
        for size in crate::theme::FONT_SCALE {
            super::super::assert_on_ruler(&[("line_height", line_height(size))]);
        }
    }
}
