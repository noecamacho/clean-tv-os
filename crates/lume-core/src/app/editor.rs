//! The home screen editor: the favorites.

use super::page::{self, LEFT, WIDTH};
use super::painter::{fade, Painter};
use super::{Anim, Effect, LumeApp, PendingFocus, Screen, Tab};
use crate::home::config::{HomeConfig, MAX_FAVORITES};
use crate::i18n::{Arg, PluralId, StringId};
use crate::icons;
use crate::render::Rect;
use crate::text::{Align, TextStyle};
use crate::theme::{vars, Color, Paint, StateFlags, THEME};
use crate::ui::focus::{FocusId, Focusable, Row};
use crate::wallpaper::WallpaperId;
use std::collections::HashMap;

pub const RESTORE: FocusId = 4001;
pub const ROUND_FAVORITES: FocusId = 4002;
pub const WALLPAPER_BASE: FocusId = 4100;
pub(super) const FAVORITE_BUTTON_BASE: FocusId = 4400;
pub(super) const ADD_APP_BASE: FocusId = 4600;
pub const CONFIRM_RESTORE: FocusId = 4900;
pub const CONFIRM_CANCEL: FocusId = 4901;

const CHIP_HEIGHT: f32 = 40.0;
const CHIP_GAP: f32 = 16.0;
/// Space inside the favorites panel, around its content.
const PANEL_PADDING_X: f32 = 32.0;
const PANEL_PADDING_Y: f32 = 24.0;
const CARD_WIDTH: f32 = 168.0;
const CARD_NAME_HEIGHT: f32 = 56.0;
const CARD_CONTROLS_HEIGHT: f32 = 60.0;
const CARD_GAP: f32 = 16.0;
/// Where the content starts: 36 below the title's baseline at 116 (no subtitle).
const TOP: f32 = 152.0;
const ROW_HEIGHT: f32 = 64.0;
const ROW_GAP: f32 = 16.0;
const WALLPAPER_CARD_WIDTH: f32 = 178.0;
const WALLPAPER_CARD_HEIGHT: f32 = 100.0;
const WALLPAPER_CARD_GAP: f32 = 16.0;
const HEADING_GAP: f32 = 32.0;
/// The line under the favorites heading that says what the panel does.
const DESCRIPTION_HEIGHT: f32 = 48.0;
const CARDS_TO_CHIPS_GAP: f32 = 20.0;
const BOTTOM_PADDING: f32 = 120.0;
const BUTTON_HEIGHT: f32 = 48.0;
/// How far below the top of the content the first line of cards reaches.
const FIRST_CARDS_DEPTH: f32 = 540.0;
const BUTTON_MIN_WIDTH: f32 = 160.0;
/// Space between the favorites panel and the buttons under it.
const BUTTONS_GAP: f32 = 24.0;

/// The buttons of the design system (see the `design-system` spec).
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub(super) enum ButtonKind {
    /// The action of the screen.
    Primary,
    /// The quiet alternative.
    Secondary,
    /// What cannot be undone.
    Danger,
}

impl ButtonKind {
    fn component(self) -> &'static crate::theme::ComponentTheme {
        match self {
            ButtonKind::Primary => &THEME.button,
            ButtonKind::Secondary => &THEME.button_secondary,
            ButtonKind::Danger => &THEME.button_danger,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct EditorState {
    /// The "Restore defaults" confirmation is open.
    pub confirm: bool,
    /// It was opened from the Home screen ("Add favorites"), not from the settings: Back goes
    /// home again instead of to the settings.
    pub from_home: bool,
    /// Current rendered horizontal scroll offset for wallpaper mini-cards.
    pub wallpaper_scroll_x: f32,
    /// Target horizontal scroll offset for wallpaper mini-cards.
    pub target_wallpaper_scroll_x: f32,
}

impl LumeApp {
    pub(super) fn open_editor(&mut self, from_home: bool) {
        self.screen = Screen::Editor;
        let wp_idx = self.config.wallpaper.index() as FocusId;
        self.editor = EditorState {
            from_home,
            ..EditorState::default()
        };
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::Id(WALLPAPER_BASE + wp_idx));
        self.focus.set_rows(vec![]);
    }

    /// Apps that are not favorites yet, in the catalog's order.
    pub(super) fn addable_apps(&self) -> Vec<(String, String)> {
        self.catalog
            .apps()
            .iter()
            .filter(|a| !self.config.favorites.contains(&a.package))
            .map(|a| (a.package.clone(), a.label.clone()))
            .collect()
    }

    pub(super) fn draw_editor(&mut self, p: &mut Painter) {
        // Two passes: the first only lays out (to know where the focus is and how far to
        // scroll), the second draws with the final scroll position.
        let layout = self.editor_pass(p, false, 0.0);
        self.update_focus(layout.rows);
        let focused = self.focus.current_id();
        let rect = focused.and_then(|id| layout.rects.get(&id).copied());
        let view_height = p.display.visible_rect().bottom().max(720.0);
        self.reveal_vertical(rect, layout.content_height, 0.0, view_height);
        if rect.is_some_and(|r| r.y < TOP + FIRST_CARDS_DEPTH) {
            // The first line of cards brings the whole page back to the top, with its header.
            self.scroll_to(0.0);
        }
        let scroll_y = self.current_scroll_y();
        self.editor_pass(p, true, scroll_y);
    }

    /// Lays out the whole screen. With `draw` it also paints it.
    fn editor_pass(&mut self, p: &mut Painter, draw: bool, scroll_y: f32) -> Layout {
        let mut layout = Layout::default();
        let t = self.translator;
        if draw {
            page::draw_header(
                p,
                &[
                    &t.get(StringId::SettingsTitle),
                    &t.get(StringId::SettingsHomeScreen),
                ],
                &t.get(StringId::SettingsHomeScreen),
                None,
                scroll_y,
            );
        }

        let wp_top = TOP;
        let panel_top = self.wallpaper_section(p, &mut layout, wp_top, scroll_y, draw);
        let y = self.favorites_panel(p, &mut layout, panel_top, scroll_y, draw);

        // "Restore defaults" at the left, on the line under the panel. The "Done" button is
        // removed: all changes persist immediately and Back is the standard Android TV exit.
        let y = y + BUTTONS_GAP;
        let restore_label = t.get(StringId::EditorRestoreDefaults).into_owned();
        let button_style = p.style(&THEME.button, StateFlags::NONE);
        let width_of = |p: &mut Painter, label: &str| {
            (p.measure(label, &button_style) + 2.0 * 28.0).max(BUTTON_MIN_WIDTH)
        };
        let restore_width = width_of(p, &restore_label);
        let restore = Rect::new(LEFT, y, restore_width, BUTTON_HEIGHT);
        layout.rects.insert(RESTORE, restore);
        layout
            .rows
            .push(Row::new(vec![Focusable::new(RESTORE, restore)]).with_entry_down(RESTORE));
        let view_height = p.display.visible_rect().bottom().max(720.0);
        let restore_scrolled = restore.translate(0.0, -scroll_y);
        let is_visible = |r: Rect| r.bottom() >= -32.0 && r.y <= view_height + 32.0;
        if draw && !self.editor.confirm && is_visible(restore_scrolled) {
            p.list.fill(
                Rect::new(LEFT, y - 12.0 - scroll_y, WIDTH, 2.0),
                1.0,
                Paint::Solid(vars::SURFACE),
            );
            self.draw_button(
                p,
                restore_scrolled,
                &restore_label,
                RESTORE,
                ButtonKind::Secondary,
            );
        }
        layout.content_height = y + BUTTON_HEIGHT + BOTTOM_PADDING;

        if self.editor.confirm {
            // The dialog traps the focus.
            let dialog = self.confirm_dialog(p, draw);
            layout.rows = dialog.0;
            layout.rects.extend(dialog.1);
        }
        layout
    }

    /// The favorites: order, remove, and add.
    fn favorites_panel(
        &mut self,
        p: &mut Painter,
        layout: &mut Layout,
        top: f32,
        scroll_y: f32,
        draw: bool,
    ) -> f32 {
        let t = self.translator;
        let inner_left = LEFT + PANEL_PADDING_X;
        let inner_width = WIDTH - 2.0 * PANEL_PADDING_X;
        let per_line =
            (((inner_width + CARD_GAP) / (CARD_WIDTH + CARD_GAP)).floor() as usize).max(1);
        let card_height = CARD_NAME_HEIGHT + CARD_CONTROLS_HEIGHT;
        let favorites = self.config.favorites.clone();
        let card_lines = favorites.len().div_ceil(per_line);

        let heading_y = top + PANEL_PADDING_Y;

        // The heading, then a line saying what the panel is for, then the cards.
        let cards_top = heading_y + HEADING_GAP + DESCRIPTION_HEIGHT;
        let mut y = cards_top;
        if card_lines > 0 {
            y += card_lines as f32 * card_height + (card_lines as f32 - 1.0) * CARD_GAP;
            y += CARDS_TO_CHIPS_GAP;
        }

        let addable = self.addable_apps();
        let full = self.config.favorites_full();
        let add_label = t.get(StringId::EditorAddApp).into_owned();
        let label_style = TextStyle::new(17.0, 500);
        let add_label_width = p.text.measure(&add_label, &label_style);
        let chip_style = TextStyle::new(18.0, 700);
        let widths: Vec<f32> = addable
            .iter()
            .map(|(_, label)| 20.0 + 16.0 + 10.0 + p.text.measure(label, &chip_style) + 20.0)
            .collect();
        let chips_x = inner_left + add_label_width + 16.0;
        let positions = flow(
            &widths,
            chips_x,
            LEFT + WIDTH - PANEL_PADDING_X,
            CHIP_GAP,
            inner_left,
            CHIP_HEIGHT + CHIP_GAP,
            y,
        );
        let chip_lines = if addable.is_empty() {
            1
        } else {
            positions.last().map_or(1, |q| q.line + 1)
        };
        let panel_bottom = y
            + chip_lines as f32 * CHIP_HEIGHT
            + (chip_lines as f32 - 1.0) * CHIP_GAP
            + PANEL_PADDING_Y;
        let panel = Rect::new(LEFT, top, WIDTH, panel_bottom - top);

        let view_height = p.display.visible_rect().bottom().max(720.0);
        let is_visible = |r: Rect| r.bottom() >= -32.0 && r.y <= view_height + 32.0;

        if draw && is_visible(panel.translate(0.0, -scroll_y)) {
            let style = p.style(&THEME.panel, StateFlags::NONE);
            p.draw_box(panel.translate(0.0, -scroll_y), &style);
            let heading = t.get(StringId::EditorFavoritesOrder).to_uppercase();
            let heading_style = p.style(&THEME.card_heading, StateFlags::NONE);
            p.text.draw(
                p.list,
                &heading,
                inner_left,
                heading_y + 17.0 - scroll_y,
                &Painter::text_style(&heading_style),
                vars::PRIMARY,
            );
            let description = t.get(StringId::EditorFavoritesDescription).into_owned();
            let description_style = TextStyle::new(16.0, 500);
            let shown = p
                .text
                .ellipsize(&description, &description_style, inner_width)
                .into_owned();
            p.text.draw(
                p.list,
                &shown,
                inner_left,
                heading_y + HEADING_GAP + 20.0 - scroll_y,
                &description_style,
                vars::TEXT_MUTED,
            );
            let hint = t.get(StringId::EditorUseArrows).into_owned();
            let hint_style = TextStyle::new(16.0, 500);
            let width = p.text.measure(&hint, &hint_style);
            p.text.draw(
                p.list,
                &hint,
                LEFT + WIDTH - PANEL_PADDING_X - width,
                heading_y + 17.0 - scroll_y,
                &hint_style,
                vars::TEXT_MUTED,
            );
        }

        // The cards, a line at a time, each with three buttons under it.
        for line in 0..card_lines {
            let mut items: Vec<Focusable> = Vec::new();
            for column in 0..per_line {
                let index = line * per_line + column;
                let Some(package) = favorites.get(index) else {
                    break;
                };
                let x = inner_left + column as f32 * (CARD_WIDTH + CARD_GAP);
                let card_y = cards_top + line as f32 * (card_height + CARD_GAP);
                let rects: [Rect; 3] = [0.0, 1.0, 2.0].map(|b| {
                    Rect::new(
                        x + 18.0 + b * 50.0,
                        card_y + CARD_NAME_HEIGHT + 6.0,
                        44.0,
                        44.0,
                    )
                });
                let button_ids: [FocusId; 3] =
                    [0, 1, 2].map(|b| FAVORITE_BUTTON_BASE + index as u32 * 3 + b);
                let last_index = favorites.len() - 1;
                for b in 0..3 {
                    layout.rects.insert(button_ids[b], rects[b]);
                    let disabled = (b == 0 && index == 0) || (b == 1 && index == last_index);
                    items.push(if disabled {
                        Focusable::disabled(button_ids[b], rects[b])
                    } else {
                        Focusable::new(button_ids[b], rects[b])
                    });
                }
                let card_scrolled = Rect::new(x, card_y - scroll_y, CARD_WIDTH, card_height);
                if draw && is_visible(card_scrolled) {
                    let (label, color) = match self.catalog.get(package) {
                        Some(app) => (app.label.clone(), app.color),
                        None => (package.clone(), vars::TILE_DEFAULT),
                    };
                    self.draw_card(
                        p,
                        card_scrolled,
                        index + 1,
                        &label,
                        color,
                        rects,
                        button_ids,
                        (index == 0, index == last_index),
                        scroll_y,
                    );
                }
            }
            layout.row(items);
        }

        // "Add app".
        let mut chip_rows: Vec<Vec<Focusable>> = vec![Vec::new(); chip_lines];
        if draw {
            let first_baseline = y + CHIP_HEIGHT / 2.0 + 6.0 - scroll_y;
            p.text.draw(
                p.list,
                &add_label,
                inner_left,
                first_baseline,
                &label_style,
                vars::TEXT_MUTED,
            );
        }
        for (i, ((_, label), pos)) in addable.iter().zip(&positions).enumerate() {
            let rect = Rect::new(pos.x, pos.y, widths[i], CHIP_HEIGHT);
            let id = ADD_APP_BASE + i as u32;
            layout.rects.insert(id, rect);
            if !full {
                chip_rows[pos.line].push(Focusable::new(id, rect));
            }
            let chip_scrolled = rect.translate(0.0, -scroll_y);
            if draw && is_visible(chip_scrolled) {
                let progress = self.focus_progress(id);
                self.draw_add_chip(p, chip_scrolled, label, progress, full);
            }
        }
        if full && draw && is_visible(panel.translate(0.0, -scroll_y)) {
            let message = t.plural(
                PluralId::EditorLimitReached,
                MAX_FAVORITES as i64,
                &[Arg::Int(MAX_FAVORITES as i64)],
            );
            let style = TextStyle::new(16.0, 500);
            let width = p.text.measure(&message, &style);
            p.text.draw(
                p.list,
                &message,
                LEFT + WIDTH - 32.0 - width,
                panel_bottom - 24.0 - 14.0 - scroll_y,
                &style,
                vars::TEXT_MUTED,
            );
        }
        for items in chip_rows {
            layout.row(items);
        }
        panel_bottom
    }

    // ---- drawing of the pieces --------------------------------------------------------------

    /// Section displaying the wallpapers as 178x100 mini-cards inside a panel with horizontal scrolling.
    fn wallpaper_section(
        &mut self,
        p: &mut Painter,
        layout: &mut Layout,
        top: f32,
        scroll_y: f32,
        draw: bool,
    ) -> f32 {
        let t = self.translator;
        let inner_left = LEFT + PANEL_PADDING_X;
        let inner_width = WIDTH - 2.0 * PANEL_PADDING_X;

        let heading_y = top + PANEL_PADDING_Y;
        let cards_y = heading_y + HEADING_GAP;
        // Under the wallpapers, a divider and the shape of the favorites.
        let div_y = cards_y + WALLPAPER_CARD_HEIGHT + ROW_GAP + 8.0;
        let round_y = div_y + ROW_GAP;
        let round_rect = Rect::new(inner_left, round_y, inner_width, ROW_HEIGHT);
        let panel_bottom = round_y + ROW_HEIGHT + PANEL_PADDING_Y;
        let panel = Rect::new(LEFT, top, WIDTH, panel_bottom - top);

        let focused_id = self.focus.current_id();
        let card_step = WALLPAPER_CARD_WIDTH + WALLPAPER_CARD_GAP;
        let max_scroll =
            (WallpaperId::ALL.len() as f32 * card_step - WALLPAPER_CARD_GAP - inner_width).max(0.0);

        let focused_wp_index = focused_id.and_then(|fid| {
            if (WALLPAPER_BASE..WALLPAPER_BASE + WallpaperId::ALL.len() as FocusId).contains(&fid) {
                Some((fid - WALLPAPER_BASE) as usize)
            } else {
                None
            }
        });

        if let Some(index) = focused_wp_index {
            let card_left = inner_left + index as f32 * card_step;
            let card_right = card_left + WALLPAPER_CARD_WIDTH;

            if card_right - self.editor.target_wallpaper_scroll_x > inner_left + inner_width {
                self.editor.target_wallpaper_scroll_x = card_right - (inner_left + inner_width);
            } else if card_left - self.editor.target_wallpaper_scroll_x < inner_left {
                self.editor.target_wallpaper_scroll_x = card_left - inner_left;
            }
        }
        self.editor.target_wallpaper_scroll_x =
            self.editor.target_wallpaper_scroll_x.clamp(0.0, max_scroll);

        if draw {
            let current_scroll = self.anim.get(
                Anim::WallpaperScrollX,
                self.editor.wallpaper_scroll_x,
                self.now_ms,
            );
            if (current_scroll - self.editor.target_wallpaper_scroll_x).abs() > 0.5 {
                self.anim.animate_to(
                    Anim::WallpaperScrollX,
                    current_scroll,
                    self.editor.target_wallpaper_scroll_x,
                    self.now_ms,
                    Some(vars::FOCUS_TIME),
                );
                self.editor.wallpaper_scroll_x = current_scroll;
            } else {
                self.editor.wallpaper_scroll_x = self.editor.target_wallpaper_scroll_x;
            }
        }

        let view_height = p.display.visible_rect().bottom().max(720.0);
        let is_visible = |r: Rect| r.bottom() >= -32.0 && r.y <= view_height + 32.0;

        if draw && !self.editor.confirm && is_visible(panel.translate(0.0, -scroll_y)) {
            let style = p.style(&THEME.panel, StateFlags::NONE);
            p.draw_box(panel.translate(0.0, -scroll_y), &style);

            let heading = t.get(StringId::EditorAppearance).to_uppercase();
            let heading_style = p.style(&THEME.card_heading, StateFlags::NONE);
            p.text.draw(
                p.list,
                &heading,
                inner_left,
                heading_y + 17.0 - scroll_y,
                &Painter::text_style(&heading_style),
                vars::PRIMARY,
            );

            let active_idx = focused_wp_index.unwrap_or_else(|| self.config.wallpaper.index());
            let total = WallpaperId::ALL.len();
            let hint = if self.editor.wallpaper_scroll_x <= 4.0 {
                format!("{} / {}  ›", active_idx + 1, total)
            } else if self.editor.wallpaper_scroll_x >= max_scroll - 4.0 {
                format!("‹  {} / {}", active_idx + 1, total)
            } else {
                format!("‹  {} / {}  ›", active_idx + 1, total)
            };
            let hint_style = TextStyle::new(16.0, 700);
            let width = p.text.measure(&hint, &hint_style);
            p.text.draw(
                p.list,
                &hint,
                LEFT + WIDTH - PANEL_PADDING_X - width,
                heading_y + 17.0 - scroll_y,
                &hint_style,
                vars::TEXT_MUTED,
            );
        }

        let mut items = Vec::with_capacity(WallpaperId::ALL.len());
        for (i, _) in WallpaperId::ALL.iter().enumerate() {
            let card_id = WALLPAPER_BASE + i as FocusId;
            let card_x = inner_left + i as f32 * card_step;
            let card_rect = Rect::new(card_x, cards_y, WALLPAPER_CARD_WIDTH, WALLPAPER_CARD_HEIGHT);
            layout.rects.insert(card_id, card_rect);
            items.push(Focusable::new(card_id, card_rect));
        }

        layout
            .rows
            .push(Row::new(items).with_entry_down(ROUND_FAVORITES));
        layout.rects.insert(ROUND_FAVORITES, round_rect);
        layout.rows.push(
            Row::new(vec![Focusable::new(ROUND_FAVORITES, round_rect)])
                .with_entry_up(WALLPAPER_BASE + self.config.wallpaper.index() as FocusId)
                .with_entry_down(FAVORITE_BUTTON_BASE),
        );

        if draw && !self.editor.confirm {
            self.draw_round_favorites_row(p, round_rect.translate(0.0, -scroll_y));
            p.list.fill(
                Rect::new(inner_left, div_y - scroll_y, inner_width, 1.0),
                0.5,
                Paint::Solid(vars::SURFACE_ROW),
            );
            let visible_clip = Rect::new(
                inner_left - 6.0,
                cards_y - 8.0 - scroll_y,
                inner_width + 12.0,
                WALLPAPER_CARD_HEIGHT + 16.0,
            );
            p.list.push_clip(visible_clip);

            for (i, &id) in WallpaperId::ALL.iter().enumerate() {
                let card_id = WALLPAPER_BASE + i as FocusId;
                let card_x = inner_left + i as f32 * card_step - self.editor.wallpaper_scroll_x;
                let card_rect = Rect::new(
                    card_x,
                    cards_y - scroll_y,
                    WALLPAPER_CARD_WIDTH,
                    WALLPAPER_CARD_HEIGHT,
                );

                if card_rect.right() >= inner_left && card_rect.x <= inner_left + inner_width {
                    self.draw_wallpaper_card(p, card_rect, id, card_id);
                }
            }

            p.list.pop_clip();

            if self.editor.wallpaper_scroll_x > 4.0 {
                let left_edge =
                    Rect::new(inner_left, cards_y - scroll_y, 48.0, WALLPAPER_CARD_HEIGHT);
                let edge_scrim = Paint::Gradient(crate::theme::Gradient::new(
                    270.0,
                    &[
                        crate::theme::GradientStop {
                            color: Color {
                                r: 18,
                                g: 28,
                                b: 37,
                                a: 220,
                            },
                            position: 0.0,
                        },
                        crate::theme::GradientStop {
                            color: Color::TRANSPARENT,
                            position: 1.0,
                        },
                    ],
                ));
                p.list.fill(left_edge, 12.0, edge_scrim);
                let chevron_rect = Rect::new(
                    inner_left + 6.0,
                    cards_y - scroll_y + (WALLPAPER_CARD_HEIGHT - 24.0) / 2.0,
                    24.0,
                    24.0,
                );
                p.icon(icons::CHEVRON_LEFT, chevron_rect, 20.0, vars::TEXT_MUTED);
            }
            if self.editor.wallpaper_scroll_x < max_scroll - 4.0 {
                let right_edge = Rect::new(
                    inner_left + inner_width - 48.0,
                    cards_y - scroll_y,
                    48.0,
                    WALLPAPER_CARD_HEIGHT,
                );
                let edge_scrim = Paint::Gradient(crate::theme::Gradient::new(
                    90.0,
                    &[
                        crate::theme::GradientStop {
                            color: Color {
                                r: 18,
                                g: 28,
                                b: 37,
                                a: 220,
                            },
                            position: 0.0,
                        },
                        crate::theme::GradientStop {
                            color: Color::TRANSPARENT,
                            position: 1.0,
                        },
                    ],
                ));
                p.list.fill(right_edge, 12.0, edge_scrim);
                let chevron_rect = Rect::new(
                    inner_left + inner_width - 30.0,
                    cards_y - scroll_y + (WALLPAPER_CARD_HEIGHT - 24.0) / 2.0,
                    24.0,
                    24.0,
                );
                p.icon(icons::CHEVRON_RIGHT, chevron_rect, 20.0, vars::TEXT_MUTED);
            }
        }

        panel_bottom + BUTTONS_GAP
    }

    fn draw_wallpaper_card(
        &mut self,
        p: &mut Painter,
        rect: Rect,
        id: WallpaperId,
        card_id: FocusId,
    ) {
        let is_focused = self.focus.current_id() == Some(card_id);
        let is_active = self.config.wallpaper == id;

        let target_rect = if is_focused {
            rect.scaled_about_center(1.05)
        } else {
            rect
        };

        if is_focused {
            p.list.shadow(
                target_rect,
                12.0,
                Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 160,
                },
                16.0,
            );
        }

        // Real thumbnail preview!
        p.list.wallpaper_thumb(target_rect, id.thumb_uv(), 12.0);

        // Scrim at the bottom for text legibility
        let scrim_rect = Rect::new(
            target_rect.x,
            target_rect.bottom() - 36.0,
            target_rect.w,
            36.0,
        );
        let scrim_paint = Paint::Gradient(crate::theme::Gradient::new(
            180.0,
            &[
                crate::theme::GradientStop {
                    color: Color::TRANSPARENT,
                    position: 0.0,
                },
                crate::theme::GradientStop {
                    color: Color {
                        r: 6,
                        g: 10,
                        b: 16,
                        a: 220,
                    },
                    position: 1.0,
                },
            ],
        ));
        p.list.fill(scrim_rect, 12.0, scrim_paint);

        // Outline / focus ring
        if is_focused {
            p.list.ring(target_rect, 12.0, 3.0, vars::PRIMARY);
        } else if is_active {
            p.list.ring(target_rect, 12.0, 2.0, vars::PRIMARY);
        } else {
            p.list.ring(target_rect, 12.0, 1.0, vars::CONTROL);
        }

        // Active indicator badge if currently selected
        if is_active {
            let badge_size = 18.0;
            let badge_rect = Rect::new(
                target_rect.right() - badge_size - 8.0,
                target_rect.y + 8.0,
                badge_size,
                badge_size,
            );
            p.list.fill(badge_rect, 9.0, Paint::Solid(vars::PRIMARY));
            let dot_rect = Rect::new(badge_rect.x + 5.0, badge_rect.y + 5.0, 8.0, 8.0);
            p.list.fill(dot_rect, 4.0, Paint::Solid(Color::WHITE));
        }

        // Name label
        let name = self.translator.get(id.title_string_id());
        let label_style = if is_focused {
            TextStyle::new(16.0, 700)
        } else {
            TextStyle::new(16.0, 500)
        };
        let label_color = if is_focused {
            vars::TEXT
        } else {
            vars::TEXT_MUTED
        };
        let text_y = target_rect.bottom() - 12.0;
        let text_x = target_rect.x + 12.0;
        p.text
            .draw(p.list, &name, text_x, text_y, &label_style, label_color);
    }

    fn draw_round_favorites_row(&mut self, p: &mut Painter, rect: Rect) {
        let t = self.translator;
        let progress = self.focus_progress(ROUND_FAVORITES);
        let style = p.focus_style(&THEME.editor_row, progress);
        p.draw_box(rect, &style);
        let label = t.get(StringId::EditorRoundFavorites);
        let text_rect = Rect::new(rect.x + 16.0, rect.y, rect.w - 16.0 - 48.0 - 16.0, rect.h);
        p.text(&label, text_rect, Align::Start, &style);

        let switch_x = rect.x + rect.w - 16.0 - 48.0;
        let switch_y = rect.y + (rect.h - 28.0) / 2.0;
        let switch_rect = Rect::new(switch_x, switch_y, 48.0, 28.0);
        let track_paint = if self.config.round_favorites {
            Paint::Solid(vars::PRIMARY)
        } else {
            Paint::Solid(vars::CONTROL)
        };
        p.list.fill(switch_rect, 14.0, track_paint);

        let knob_x = if self.config.round_favorites {
            switch_x + 24.0
        } else {
            switch_x + 4.0
        };
        let knob_y = switch_y + 4.0;
        let knob_rect = Rect::new(knob_x, knob_y, 20.0, 20.0);
        let knob_color = if self.config.round_favorites {
            vars::ON_PRIMARY
        } else {
            vars::TEXT_MUTED
        };
        p.list.fill(knob_rect, 10.0, Paint::Solid(knob_color));
    }

    /// A button of the design system: `kind` picks the component of the theme.
    pub(super) fn draw_button(
        &mut self,
        p: &mut Painter,
        rect: Rect,
        label: &str,
        id: FocusId,
        kind: ButtonKind,
    ) {
        let progress = self.focus_progress(id);
        let style = p.focus_style(kind.component(), progress);
        p.draw_box(rect, &style);
        p.text(
            label,
            rect.scaled_about_center(style.scale),
            Align::Center,
            &style,
        );
    }

    fn draw_round_button(
        &mut self,
        p: &mut Painter,
        rect: Rect,
        icon: char,
        progress: f32,
        disabled: bool,
        danger: bool,
    ) {
        let state = if disabled {
            StateFlags::DISABLED
        } else {
            StateFlags::NONE
        };
        let mut style = if disabled {
            p.style(&THEME.icon_button, state)
        } else {
            p.focus_style(&THEME.icon_button, progress)
        };
        style.border_radius = rect.w / 2.0;
        style.background = Some(Paint::Solid(match (disabled, danger) {
            (true, _) => vars::CONTROL_DISABLED,
            (false, true) => vars::DANGER_BG,
            (false, false) => vars::CONTROL,
        }));
        p.draw_box(rect, &style);
        let color = match (disabled, danger) {
            (true, _) => vars::TEXT_DISABLED,
            (false, true) => vars::DANGER_FG,
            (false, false) => vars::TEXT,
        };
        p.icon(icon, rect, 24.0, color);
    }

    fn draw_add_chip(
        &mut self,
        p: &mut Painter,
        rect: Rect,
        label: &str,
        progress: f32,
        disabled: bool,
    ) {
        let radius = rect.h / 2.0;
        let opacity = if disabled { 0.4 } else { 1.0 };
        p.list
            .fill(rect, radius, Paint::Solid(fade(vars::SURFACE, opacity)));
        p.list.ring(rect, radius, 2.0, fade(vars::CONTROL, opacity));
        p.icon(
            icons::PLUS,
            Rect::new(rect.x + 18.0, rect.y, 16.0, rect.h),
            16.0,
            fade(vars::TEXT, opacity),
        );
        let style = TextStyle::new(18.0, 700);
        let text_rect = Rect::new(
            rect.x + 18.0 + 16.0 + 10.0,
            rect.y,
            rect.w - 18.0 - 16.0 - 10.0 - 16.0,
            rect.h,
        );
        p.text_with(
            label,
            text_rect,
            Align::Start,
            &style,
            fade(vars::TEXT, opacity),
        );
        if !disabled {
            self.draw_focus_ring(p, rect, radius, progress);
        }
    }

    fn draw_focus_ring(&mut self, p: &mut Painter, rect: Rect, radius: f32, progress: f32) {
        if progress <= 0.0 {
            return;
        }
        let ring = rect.grow(3.0);
        p.list.ring(
            ring,
            radius + 3.0,
            3.0,
            Color {
                a: (255.0 * progress) as u8,
                ..vars::PRIMARY
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_card(
        &mut self,
        p: &mut Painter,
        card: Rect,
        number: usize,
        label: &str,
        color: Color,
        buttons: [Rect; 3],
        button_ids: [FocusId; 3],
        (first, last): (bool, bool),
        scroll_y: f32,
    ) {
        let name = Rect::new(card.x, card.y, card.w, CARD_NAME_HEIGHT);
        p.list.fill(name, 12.0, Paint::Solid(color));
        // Square off the bottom of the colored part where it meets the controls.
        p.list.fill(
            Rect::new(card.x, name.bottom() - 12.0, card.w, 12.0),
            0.0,
            Paint::Solid(color),
        );
        let controls = Rect::new(card.x, name.bottom(), card.w, CARD_CONTROLS_HEIGHT);
        p.list.fill(controls, 12.0, Paint::Solid(vars::SURFACE_ROW));
        p.list.fill(
            Rect::new(card.x, controls.y, card.w, 12.0),
            0.0,
            Paint::Solid(vars::SURFACE_ROW),
        );
        let number_style = TextStyle::new(16.0, 500);
        p.text_with(
            &number.to_string(),
            Rect::new(name.x + 14.0, name.y, 20.0, name.h),
            Align::Start,
            &number_style,
            fade(Color::WHITE, 0.7),
        );
        let label_style = TextStyle::new(20.0, 700);
        let label_rect = Rect::new(name.x + 36.0, name.y, name.w - 36.0 - 10.0, name.h);
        p.text_with(label, label_rect, Align::Start, &label_style, Color::WHITE);
        let button_icons = [icons::CHEVRON_LEFT, icons::CHEVRON_RIGHT, icons::CLOSE];
        for b in 0..3 {
            let disabled = (b == 0 && first) || (b == 1 && last);
            let progress = self.focus_progress(button_ids[b]);
            let rect = buttons[b].translate(0.0, -scroll_y);
            self.draw_round_button(p, rect, button_icons[b], progress, disabled, b == 2);
        }
    }

    /// The "Restore defaults?" dialog; returns its focus rows and rectangles.
    fn confirm_dialog(
        &mut self,
        p: &mut Painter,
        draw: bool,
    ) -> (Vec<Row>, HashMap<FocusId, Rect>) {
        let t = self.translator;
        let visible = p.display.visible_rect();
        let width = 640.0;
        let height = 280.0;
        let dialog = Rect::new(640.0 - width / 2.0, 360.0 - height / 2.0, width, height);
        let confirm_label = t.get(StringId::EditorRestoreConfirm).into_owned();
        let cancel_label = t.get(StringId::EditorCancel).into_owned();
        let button_style = p.style(&THEME.button, StateFlags::NONE);
        let confirm_w = (p.measure(&confirm_label, &button_style) + 56.0).max(140.0);
        let cancel_w = (p.measure(&cancel_label, &button_style) + 56.0).max(140.0);
        let y = dialog.bottom() - 32.0 - BUTTON_HEIGHT;
        let cancel = Rect::new(dialog.right() - 32.0 - cancel_w, y, cancel_w, BUTTON_HEIGHT);
        let confirm = Rect::new(cancel.x - 16.0 - confirm_w, y, confirm_w, BUTTON_HEIGHT);
        if draw {
            p.list.fill(
                visible,
                0.0,
                Paint::Solid(Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 170,
                }),
            );
            p.list.fill(
                dialog.translate(0.0, 12.0).grow(2.0),
                24.0,
                Paint::Solid(Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 60,
                }),
            );
            p.list.fill(dialog, 20.0, Paint::Solid(vars::SURFACE_ROW));
            p.list.ring(dialog, 20.0, 2.0, vars::BORDER);
            let title_style = TextStyle::new(28.0, 800);
            let title = p
                .text
                .ellipsize(
                    &t.get(StringId::EditorRestoreTitle),
                    &title_style,
                    width - 64.0,
                )
                .into_owned();
            p.text.draw(
                p.list,
                &title,
                dialog.x + 32.0,
                dialog.y + 66.0,
                &title_style,
                vars::TEXT,
            );
            let body_style = TextStyle::new(18.0, 500);
            let body = t.get(StringId::EditorRestoreBody).into_owned();
            let mut line_y = dialog.y + 110.0;
            for line in wrap(p, &body, &body_style, width - 64.0) {
                p.text.draw(
                    p.list,
                    &line,
                    dialog.x + 32.0,
                    line_y,
                    &body_style,
                    vars::TEXT_MUTED,
                );
                line_y += 26.0;
            }
            self.draw_button(
                p,
                confirm,
                &confirm_label,
                CONFIRM_RESTORE,
                ButtonKind::Danger,
            );
            self.draw_button(
                p,
                cancel,
                &cancel_label,
                CONFIRM_CANCEL,
                ButtonKind::Secondary,
            );
        }
        let rows = vec![Row::new(vec![
            Focusable::new(CONFIRM_RESTORE, confirm),
            Focusable::new(CONFIRM_CANCEL, cancel),
        ])];
        let rects = HashMap::from([(CONFIRM_RESTORE, confirm), (CONFIRM_CANCEL, cancel)]);
        (rows, rects)
    }

    // ---- actions ----------------------------------------------------------------------------

    pub(super) fn activate_editor(&mut self, id: FocusId) -> Vec<Effect> {
        let save = vec![Effect::SaveConfig];
        match id {
            ROUND_FAVORITES => {
                self.config.round_favorites = !self.config.round_favorites;
                return save;
            }
            RESTORE => {
                self.editor.confirm = true;
                self.pending_focus = Some(PendingFocus::Id(CONFIRM_CANCEL));
                return vec![];
            }
            CONFIRM_RESTORE => {
                self.config = HomeConfig::first_use(&self.catalog);
                self.editor.confirm = false;
                self.pending_focus = Some(PendingFocus::Id(RESTORE));
                return save;
            }
            CONFIRM_CANCEL => {
                self.editor.confirm = false;
                self.pending_focus = Some(PendingFocus::Id(RESTORE));
                return vec![];
            }
            _ => {}
        }
        if (WALLPAPER_BASE..WALLPAPER_BASE + WallpaperId::ALL.len() as FocusId).contains(&id) {
            let index = (id - WALLPAPER_BASE) as usize;
            let chosen = WallpaperId::ALL[index];
            if self.config.wallpaper != chosen {
                self.config.wallpaper = chosen;
                return save;
            }
            return vec![];
        }
        if (FAVORITE_BUTTON_BASE..ADD_APP_BASE).contains(&id) {
            let offset = id - FAVORITE_BUTTON_BASE;
            let (index, button) = ((offset / 3) as usize, offset % 3);
            let changed = match button {
                0 => self.config.move_favorite_left(index),
                1 => self.config.move_favorite_right(index),
                _ => self.config.remove_favorite(index),
            };
            if changed {
                let target = match button {
                    0 => FAVORITE_BUTTON_BASE + (index as u32 - 1) * 3,
                    1 => FAVORITE_BUTTON_BASE + (index as u32 + 1) * 3 + 1,
                    _ => id,
                };
                self.pending_focus = Some(PendingFocus::Id(target));
                return save;
            }
            return vec![];
        }
        if (ADD_APP_BASE..CONFIRM_RESTORE).contains(&id) {
            let addable = self.addable_apps();
            let Some((package, _)) = addable.get((id - ADD_APP_BASE) as usize) else {
                return vec![];
            };
            if self.config.add_favorite(package) {
                return save;
            }
        }
        vec![]
    }

    /// "Done", and Back at the top level of the editor.
    pub(super) fn finish_editing(&mut self) -> Vec<Effect> {
        self.screen = Screen::Main(Tab::Home);
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::FirstContent);
        vec![Effect::SaveConfig]
    }

    pub(super) fn back_from_editor(&mut self) -> Vec<Effect> {
        if self.editor.confirm {
            self.editor.confirm = false;
            self.pending_focus = Some(PendingFocus::Id(RESTORE));
            return vec![];
        }
        if self.editor.from_home {
            return self.finish_editing();
        }
        // One step back: the Lume settings, on the item that opened this screen. Back from there
        // goes on to the "Lume" tile, so the way out retraces the way in.
        self.screen = Screen::Settings;
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::Id(super::settings::ITEM_HOME_SCREEN));
        vec![Effect::SaveConfig]
    }
}

/// The rows and rectangles of the editor, collected while laying it out.
#[derive(Default)]
struct Layout {
    rows: Vec<Row>,
    rects: HashMap<FocusId, Rect>,
    content_height: f32,
}

impl Layout {
    /// Adds a row of focusable elements (empty rows are dropped).
    fn row(&mut self, items: Vec<Focusable>) {
        if !items.is_empty() {
            self.rows.push(Row::new(items));
        }
    }
}

struct Placed {
    x: f32,
    y: f32,
    line: usize,
}

/// Places items of the given `widths` left to right, wrapping at `max_x`. The first line starts
/// at `first_x`; later lines start at `next_x`.
fn flow(
    widths: &[f32],
    first_x: f32,
    max_x: f32,
    gap: f32,
    next_x: f32,
    line_pitch: f32,
    top: f32,
) -> Vec<Placed> {
    let mut placed = Vec::with_capacity(widths.len());
    let (mut x, mut line) = (first_x, 0usize);
    for &width in widths {
        if x + width > max_x && x > if line == 0 { first_x } else { next_x } {
            line += 1;
            x = next_x;
        }
        placed.push(Placed {
            x,
            y: top + line as f32 * line_pitch,
            line,
        });
        x += width + gap;
    }
    placed
}

/// Breaks `text` into lines no wider than `max_width`.
pub(super) fn wrap(p: &mut Painter, text: &str, style: &TextStyle, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if current.is_empty() || p.text.measure(&candidate, style) <= max_width {
            current = candidate;
        } else {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_is_on_the_ruler() {
        super::super::assert_on_ruler(&[
            ("CHIP_HEIGHT", CHIP_HEIGHT),
            ("CHIP_GAP", CHIP_GAP),
            ("PANEL_PADDING_X", PANEL_PADDING_X),
            ("PANEL_PADDING_Y", PANEL_PADDING_Y),
            ("CARD_WIDTH", CARD_WIDTH),
            ("CARD_NAME_HEIGHT", CARD_NAME_HEIGHT),
            ("CARD_CONTROLS_HEIGHT", CARD_CONTROLS_HEIGHT),
            ("CARD_GAP", CARD_GAP),
            ("BOTTOM_PADDING", BOTTOM_PADDING),
            ("BUTTON_HEIGHT", BUTTON_HEIGHT),
            ("FIRST_CARDS_DEPTH", FIRST_CARDS_DEPTH),
            ("BUTTON_MIN_WIDTH", BUTTON_MIN_WIDTH),
            ("BUTTONS_GAP", BUTTONS_GAP),
            ("ROW_HEIGHT", ROW_HEIGHT),
            ("ROW_GAP", ROW_GAP),
            ("HEADING_GAP", HEADING_GAP),
            ("CARDS_TO_CHIPS_GAP", CARDS_TO_CHIPS_GAP),
            ("TOP", TOP),
        ]);
    }
}
