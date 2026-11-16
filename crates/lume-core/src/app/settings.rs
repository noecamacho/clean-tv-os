//! The lume settings screen and the About screen.

use super::page::{self, LEFT, WIDTH};
use super::painter::Painter;
use super::{ids, Effect, LumeApp, PendingFocus, Screen, Tab};
use crate::device_info::SECTIONS;
use crate::i18n::{Arg, StringId};
use crate::icons;
use crate::licenses;
use crate::render::Rect;
use crate::text::Align;
use crate::text::TextStyle;
use crate::theme::{vars, ComputedStyle, StateFlags, THEME};
use crate::theme::{Color, Paint};
use crate::ui::focus::{Focusable, Row};

/// Where the source code lives, shown on the About screen.
pub const SOURCE_URL: &str = env!("CARGO_PKG_REPOSITORY");

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

pub(super) const ITEM_HOME_SCREEN: u32 = ids::SETTINGS_BASE;
pub(super) const ITEM_KEY_MAPPING: u32 = ids::SETTINGS_BASE + 1;
pub(super) const ITEM_TV_INFO: u32 = ids::SETTINGS_BASE + 2;
pub(super) const ITEM_ABOUT: u32 = ids::SETTINGS_BASE + 3;
pub(super) const ITEM_WALLPAPER: u32 = ids::SETTINGS_BASE + 4;
const ITEM_PICTURE: u32 = ids::SETTINGS_BASE + 5;
const ITEM_SOUND: u32 = ids::SETTINGS_BASE + 6;
const ITEM_NETWORK: u32 = ids::SETTINGS_BASE + 7;
const ITEM_ANDROID: u32 = ids::SETTINGS_BASE + 8;
const HOME_SIX: u32 = ids::SETTINGS_BASE + 20;
const HOME_AMBIENT: u32 = ids::SETTINGS_BASE + 22;
/// What OK on the ambient row goes through, in minutes; zero is off.
const AMBIENT_STEPS: [u16; 4] = [5, 10, 30, 0];
pub(super) const HOME_ORDER: u32 = ids::SETTINGS_BASE + 21;
const WALLPAPER_ROW: u32 = ids::SETTINGS_BASE + 40;

/// The settings are laid out as on tvOS: the title centered, an emblem of glass at the left and
/// the rows at the right, on the 1280-unit canvas.
const LIST_X: f32 = 674.0;
const LIST_WIDTH: f32 = 532.0;
const LIST_TOP: f32 = 133.0;
const ROW_HEIGHT: f32 = 48.0;
const ROW_PITCH: f32 = 56.0;
const ROW_RADIUS: f32 = 15.0;
/// What a heading adds above the first row of its section.
const SECTION_GAP: f32 = 34.0;
const EMBLEM: Rect = Rect::new(208.0, 190.0, 252.0, 252.0);

/// Which list of the settings is on screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SettingsPage {
    #[default]
    Root,
    HomeScreen,
    Wallpaper,
}

/// What a row shows at its right end.
enum Mark {
    /// It opens another screen.
    Chevron,
    Value(String),
    Switch(bool),
    /// The chosen one of a list of choices.
    Check(bool),
}

struct ListRow {
    id: u32,
    label: String,
    mark: Mark,
    /// The heading of the section this row starts.
    section: Option<String>,
}

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
        self.settings_page = SettingsPage::Root;
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::FirstContent);
        // The settings screen has no top bar: its first row is the first item.
        self.focus.set_rows(vec![]);
    }

    /// The wallpaper the screen shows: on the list of wallpapers, the one with the focus, so the
    /// whole screen is its preview.
    pub(super) fn wallpaper_on_screen(&self) -> crate::wallpaper::WallpaperId {
        let previewed = (self.screen == Screen::Settings
            && self.settings_page == SettingsPage::Wallpaper)
            .then(|| self.focus.current_id())
            .flatten()
            .and_then(|id| id.checked_sub(WALLPAPER_ROW))
            .and_then(|index| {
                crate::wallpaper::WallpaperId::ALL
                    .get(index as usize)
                    .copied()
            });
        previewed.unwrap_or(self.config.wallpaper)
    }

    /// Whether the picture behind the settings is the preview of a wallpaper, left undimmed.
    pub(super) fn previewing_wallpaper(&self) -> bool {
        self.screen == Screen::Settings && self.settings_page == SettingsPage::Wallpaper
    }

    fn settings_rows(&self) -> Vec<ListRow> {
        let t = self.translator;
        let row = |id, label: StringId, mark| ListRow {
            id,
            label: t.get(label).into_owned(),
            mark,
            section: None,
        };
        match self.settings_page {
            SettingsPage::Root => {
                let wallpaper = t.get(self.config.wallpaper.title_string_id()).into_owned();
                let mut rows = vec![
                    row(
                        ITEM_HOME_SCREEN,
                        StringId::SettingsHomeScreen,
                        Mark::Chevron,
                    ),
                    row(
                        ITEM_WALLPAPER,
                        StringId::SettingsWallpaper,
                        Mark::Value(wallpaper),
                    ),
                    row(
                        ITEM_KEY_MAPPING,
                        StringId::SettingsKeyMapping,
                        Mark::Chevron,
                    ),
                    row(ITEM_TV_INFO, StringId::SettingsTvInfo, Mark::Chevron),
                    row(ITEM_ABOUT, StringId::SettingsAbout, Mark::Chevron),
                    row(ITEM_PICTURE, StringId::ControlPicture, Mark::Chevron),
                    row(ITEM_SOUND, StringId::ControlSound, Mark::Chevron),
                    row(ITEM_NETWORK, StringId::SettingsNetwork, Mark::Chevron),
                    row(ITEM_ANDROID, StringId::SettingsAll, Mark::Chevron),
                ];
                rows[5].section = Some(t.get(StringId::SettingsTvSection).into_owned());
                rows
            }
            SettingsPage::HomeScreen => vec![
                row(
                    HOME_SIX,
                    StringId::EditorSixColumns,
                    Mark::Switch(self.config.six_columns),
                ),
                row(HOME_ORDER, StringId::SettingsHomeOrder, Mark::Chevron),
                row(
                    HOME_AMBIENT,
                    StringId::ControlAmbient,
                    Mark::Value(match self.config.ambient_minutes {
                        0 => t.get(StringId::ControlTimerOff).into_owned(),
                        minutes => format!("{minutes} min"),
                    }),
                ),
            ],
            SettingsPage::Wallpaper => crate::wallpaper::WallpaperId::ALL
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    row(
                        WALLPAPER_ROW + index as u32,
                        id.title_string_id(),
                        Mark::Check(*id == self.config.wallpaper),
                    )
                })
                .collect(),
        }
    }

    pub(super) fn draw_settings(&mut self, p: &mut Painter) {
        let t = self.translator;
        let rows = self.settings_rows();
        let mut y = LIST_TOP;
        let rects: Vec<Rect> = rows
            .iter()
            .map(|row| {
                if row.section.is_some() {
                    y += SECTION_GAP;
                }
                let rect = Rect::new(LIST_X, y, LIST_WIDTH, ROW_HEIGHT);
                y += ROW_PITCH;
                rect
            })
            .collect();
        self.update_focus(
            rows.iter()
                .zip(&rects)
                .map(|(row, rect)| Row::new(vec![Focusable::new(row.id, *rect)]))
                .collect(),
        );

        let (title, icon) = match self.settings_page {
            SettingsPage::Root => (StringId::SettingsTitle, icons::SETTINGS),
            SettingsPage::HomeScreen => (StringId::SettingsHomeScreen, icons::CONTROL_CENTER),
            SettingsPage::Wallpaper => (StringId::SettingsWallpaper, icons::PICTURE),
        };
        let soft = Color {
            r: 255,
            g: 255,
            b: 255,
            a: 170,
        };
        p.text_with(
            &t.get(title),
            Rect::new(0.0, 36.0, 1280.0, 44.0),
            Align::Center,
            &TextStyle {
                size: 30.0,
                weight: 700,
                letter_spacing: 0.0,
            },
            soft,
        );
        let small = TextStyle {
            size: 16.0,
            weight: 500,
            letter_spacing: 0.0,
        };
        if self.settings_page == SettingsPage::Wallpaper {
            // The screen itself is the preview: the left side stays clear, with only its name.
            let focused = self.wallpaper_on_screen();
            p.text_with(
                &t.get(focused.title_string_id()),
                Rect::new(54.0, 620.0, 560.0, 40.0),
                Align::Start,
                &TextStyle {
                    size: 28.0,
                    weight: 700,
                    letter_spacing: 0.0,
                },
                Color::WHITE,
            );
        } else {
            p.wallpaper_glass(EMBLEM, 56.0, self.config.wallpaper);
            p.list.fill(
                EMBLEM,
                56.0,
                Paint::Solid(Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 28,
                }),
            );
            p.list.ring(EMBLEM, 56.0, 1.0, Color { a: 70, ..soft });
            p.icon(icon, EMBLEM, 112.0, Color { a: 215, ..soft });
            // The installed version, where tvOS describes the row that has the focus.
            let version = t.format(
                StringId::AboutVersion,
                &[Arg::Str(&lume_version::display())],
            );
            p.text_with(
                &version,
                Rect::new(
                    EMBLEM.x - 120.0,
                    EMBLEM.bottom() + 28.0,
                    EMBLEM.w + 240.0,
                    24.0,
                ),
                Align::Center,
                &small,
                soft,
            );
        }

        let dark = Color {
            r: 16,
            g: 22,
            b: 32,
            a: 255,
        };
        for (row, at) in rows.iter().zip(&rects) {
            if let Some(section) = &row.section {
                p.text_with(
                    &section.to_uppercase(),
                    Rect::new(at.x + 20.0, at.y - 30.0, at.w, 22.0),
                    Align::Start,
                    &TextStyle {
                        size: 13.0,
                        weight: 700,
                        letter_spacing: 0.8,
                    },
                    Color { a: 150, ..soft },
                );
            }
            let focus = self.focus_progress(row.id).clamp(0.0, 1.0);
            let rect = at.scaled_about_center(1.0 + 0.03 * focus);
            if focus > 0.0 {
                p.list.shadow(
                    rect.translate(0.0, 8.0),
                    ROW_RADIUS,
                    Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: (110.0 * focus) as u8,
                    },
                    18.0,
                );
            }
            // Dark glass at rest; the row with the focus turns white with dark content.
            p.list.fill(
                rect,
                ROW_RADIUS,
                Paint::Solid(crate::ui::style::mix_color(
                    Color {
                        r: 14,
                        g: 20,
                        b: 30,
                        a: if self.more_contrast { 235 } else { 150 },
                    },
                    Color {
                        r: 246,
                        g: 248,
                        b: 252,
                        a: 245,
                    },
                    focus,
                )),
            );
            let ink = crate::ui::style::mix_color(Color::WHITE, dark, focus);
            let muted = Color { a: 150, ..ink };
            p.text_with(
                &row.label,
                Rect::new(rect.x + 20.0, rect.y, rect.w - 150.0, rect.h),
                Align::Start,
                &TextStyle {
                    size: 18.0,
                    weight: 500,
                    letter_spacing: 0.0,
                },
                ink,
            );
            let end = Rect::new(rect.right() - 44.0, rect.y, 28.0, rect.h);
            match &row.mark {
                Mark::Chevron => p.icon(icons::CHEVRON_RIGHT, end, 18.0, muted),
                Mark::Value(value) => {
                    p.text_with(
                        value,
                        Rect::new(rect.right() - 250.0, rect.y, 200.0, rect.h),
                        Align::End,
                        &small,
                        muted,
                    );
                    p.icon(icons::CHEVRON_RIGHT, end, 18.0, muted);
                }
                Mark::Check(true) => p.icon(icons::CHECK, end, 20.0, ink),
                Mark::Check(false) => {}
                Mark::Switch(on) => {
                    let track = Rect::new(rect.right() - 62.0, rect.y + 11.0, 44.0, 26.0);
                    let color = if *on {
                        Color {
                            r: 48,
                            g: 209,
                            b: 88,
                            a: 255,
                        }
                    } else {
                        Color { a: 70, ..ink }
                    };
                    p.list.fill(track, 13.0, Paint::Solid(color));
                    let knob = Rect::new(
                        track.x + if *on { 21.0 } else { 3.0 },
                        track.y + 3.0,
                        20.0,
                        20.0,
                    );
                    p.list.fill(knob, 10.0, Paint::Solid(Color::WHITE));
                }
            }
        }
    }

    pub(super) fn activate_settings(&mut self, id: u32) -> Vec<Effect> {
        match id {
            ITEM_HOME_SCREEN => {
                self.settings_page = SettingsPage::HomeScreen;
                self.pending_focus = Some(PendingFocus::Id(HOME_SIX));
                vec![]
            }
            ITEM_WALLPAPER => {
                self.settings_page = SettingsPage::Wallpaper;
                self.pending_focus = Some(PendingFocus::Id(
                    WALLPAPER_ROW + self.config.wallpaper.index() as u32,
                ));
                vec![]
            }
            HOME_SIX => {
                self.config.six_columns = !self.config.six_columns;
                vec![Effect::SaveConfig]
            }
            HOME_ORDER => {
                self.open_editor(false);
                vec![]
            }
            HOME_AMBIENT => {
                let at = AMBIENT_STEPS
                    .iter()
                    .position(|minutes| *minutes == self.config.ambient_minutes)
                    .unwrap_or(0);
                self.config.ambient_minutes = AMBIENT_STEPS[(at + 1) % AMBIENT_STEPS.len()];
                vec![Effect::SaveConfig]
            }
            ITEM_PICTURE => vec![Effect::OpenTvSetting {
                setting: crate::keys::SettingShortcut::Picture,
            }],
            ITEM_SOUND => vec![Effect::OpenTvSetting {
                setting: crate::keys::SettingShortcut::Sound,
            }],
            ITEM_NETWORK => vec![Effect::OpenTvSetting {
                setting: crate::keys::SettingShortcut::Wifi,
            }],
            ITEM_ANDROID => vec![Effect::OpenAndroidSettings],
            id if id >= WALLPAPER_ROW => {
                match crate::wallpaper::WallpaperId::ALL.get((id - WALLPAPER_ROW) as usize) {
                    Some(wallpaper) => {
                        self.config.wallpaper = *wallpaper;
                        vec![Effect::SaveConfig]
                    }
                    None => vec![],
                }
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
        // A page of the settings goes back to their list, on the row that opened it.
        let opened_by = match self.settings_page {
            SettingsPage::Root => None,
            SettingsPage::HomeScreen => Some(ITEM_HOME_SCREEN),
            SettingsPage::Wallpaper => Some(ITEM_WALLPAPER),
        };
        if let Some(row) = opened_by {
            self.settings_page = SettingsPage::Root;
            self.pending_focus = Some(PendingFocus::Id(row));
            return;
        }
        self.screen = Screen::Main(Tab::Home);
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::FirstContent);
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
        ]);
        for size in crate::theme::FONT_SCALE {
            super::super::assert_on_ruler(&[("line_height", line_height(size))]);
        }
    }
}
