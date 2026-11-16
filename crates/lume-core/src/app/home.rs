//! Home, laid out like tvOS: at rest the wallpaper is sharp and whole, with only the first row
//! of favorites on a glass platter against the bottom edge. Down from that row unfolds the
//! library: every row comes up at once, the platter with its row to the top, over the picture
//! out of focus. The focused banner grows over a soft shadow and, in the library, shows its
//! name under it.

use super::context_menu::Origin;
use super::dock::DockItem;
use super::painter::Painter;
use super::{ids, Anim, Effect, LumeApp};
use crate::render::Rect;
use crate::text::{Align, TextStyle};
use crate::theme::{Color, Gradient, GradientStop, Paint, THEME};
use crate::ui::focus::{Focusable, Row};

/// tvOS's five-column grid (320 pt columns, 40 pt gutters, 80 pt side margins on a 1920 pt
/// screen) on the 1280-unit canvas, with 16:9 banners, which is what Android TV apps ship.
const COLUMNS: usize = 5;
const LEFT: f32 = 54.0;
/// At rest the platter of the first row ends 19 units from the bottom edge.
const TOP: f32 = 561.0;
/// Where the first row sits once the library is unfolded.
const UNFOLDED_TOP: f32 = 65.0;
pub(super) const WIDTH: f32 = 212.0;
const HEIGHT: f32 = 120.0;
const PITCH_X: f32 = 240.0;
const PITCH_Y: f32 = 184.0;
/// How far the glass platter of the first row reaches around its banners.
const PLATTER: f32 = 20.0;
/// Concentric with the banners: their radius plus the distance to them.
const PLATTER_RADIUS: f32 = 12.0 + PLATTER;
/// The status capsule in the top right corner, inside the side margin of the grid.
/// Long enough for any move; it is cut short when the move ends.
const JIGGLE: crate::theme::Transition = crate::theme::Transition {
    duration_ms: 600_000.0,
    curve: crate::theme::Curve::Linear,
};
const STATUS: Rect = Rect::new(1280.0 - LEFT - 150.0, 28.0, 150.0, 48.0);

/// The banner at `index`. With `six` per row they are tvOS's default size (173 pt columns on
/// its canvas scaled to ours); five per row is its Large.
fn tile_rect(index: usize, six: bool) -> Rect {
    let (columns, top, width, height, pitch_x, pitch_y) = if six {
        (6, 584.0, 172.0, 97.0, 200.0, 160.0)
    } else {
        (COLUMNS, TOP, WIDTH, HEIGHT, PITCH_X, PITCH_Y)
    };
    Rect::new(
        LEFT + (index % columns) as f32 * pitch_x,
        top + (index / columns) as f32 * pitch_y,
        width,
        height,
    )
}

/// Whether a favorite is being moved or a menu is open: Home does not settle under them.
fn jiggling_now(app: &LumeApp) -> bool {
    app.dock_move.is_some() || app.menu_is_open()
}

impl LumeApp {
    /// Favorites lead; the rest of the installed catalog follows without duplicates.
    fn home_items(&self) -> Vec<DockItem> {
        let mut items = self.dock_items();
        for (index, app) in self.catalog.apps().iter().enumerate() {
            if items
                .iter()
                .any(|item| item.package.as_deref() == Some(app.package.as_str()))
            {
                continue;
            }
            items.push(DockItem {
                id: ids::GRID_BASE + index as u32,
                package: Some(app.package.clone()),
                label: app.label.clone(),
                color: app.color,
            });
        }
        items
    }

    pub(super) fn draw_home(&mut self, p: &mut Painter) {
        let items = self.home_items();
        // tvOS has no bar of tabs: only the status in the top right corner, which takes the focus
        // with Up and opens the Control Center.
        let mut rows = vec![Row::new(vec![Focusable::new(ids::CONTROL_CENTER, STATUS)])];
        let six = self.config.six_columns;
        let columns = if six { 6 } else { COLUMNS };
        let tile_rect = |index: usize| tile_rect(index, six);
        for (line_index, line) in items.chunks(columns).enumerate() {
            let mut row = Row::new(
                line.iter()
                    .enumerate()
                    .map(|(col, item)| {
                        Focusable::new(item.id, tile_rect(line_index * columns + col))
                    })
                    .collect(),
            );
            if line_index == 0 {
                row = row.with_entry_down(line[0].id);
            }
            rows.push(row);
        }
        // Under the Control Center Home is only a picture: the focus is on the controls.
        let interactive = self.screen == super::Screen::Main(super::Tab::Home);
        if interactive {
            self.update_focus(rows);
        }

        let focused_id = self.focus.current_id();
        // The way the focus last moved: the banner that takes it leans that way for a moment,
        // which stands in for the tvOS tilt.
        let now_focus = self.focus.current().map(|f| (f.id, f.rect));
        if let (Some((old_id, old)), Some((id, new))) = (self.home_focus, now_focus) {
            if old_id != id {
                let (dx, dy) = (
                    new.center().0 - old.center().0,
                    new.center().1 - old.center().1,
                );
                let length = (dx * dx + dy * dy).sqrt().max(1.0);
                self.home_sway = (dx / length, dy / length);
            }
        }
        if interactive {
            self.home_focus = now_focus;
        }
        // Two places, as on tvOS: the first row at rest against the bottom edge, or the library
        // unfolded with that row at the top. Deeper rows scroll one row at a time.
        let bottom = p.display.visible_rect().bottom();
        let (top, pitch) = (tile_rect(0).y, tile_rect(columns).y - tile_rect(0).y);
        let row = focused_id
            .and_then(|id| items.iter().position(|item| item.id == id))
            .map_or(0, |index| index / columns);
        // A row fits with the name under its banner.
        let fits = ((bottom - UNFOLDED_TOP - tile_rect(0).h - 52.0) / pitch).floor();
        self.scroll_to(if row == 0 {
            0.0
        } else {
            top - UNFOLDED_TOP + (row as f32 - fits).max(0.0) * pitch
        });
        let scroll_y = self.current_scroll_y();
        // How far the library is unfolded, for what only one of the two places shows.
        let unfolded = (scroll_y / 60.0).clamp(0.0, 1.0);

        // After a while without keys Home settles, as tvOS does: what has the focus grows a little
        // and the rest dims.
        if !self.idle
            && !jiggling_now(self)
            && self.now_ms.saturating_sub(self.last_input_ms) >= super::IDLE_AFTER_MS
        {
            self.idle = true;
            self.anim
                .animate_to(Anim::Idle, 0.0, 1.0, self.now_ms, Some(super::IDLE_IN));
        }
        let settled = self.anim.get(Anim::Idle, 0.0, self.now_ms);
        let press = self.anim.get(Anim::Press, 0.0, self.now_ms);

        // While a favorite is being moved the favorites jiggle, as icons do in the tvOS edit mode.
        let favorites = self.config.favorites.len();
        let calm = self.reduce_motion;
        let jiggling = self.dock_move.is_some() && !calm;
        if jiggling {
            if !self
                .anim
                .is_animating_where(self.now_ms, |key| matches!(key, Anim::Jiggle))
            {
                self.anim.set(Anim::Jiggle, 0.0);
                self.anim
                    .animate_to(Anim::Jiggle, 0.0, 1.0, self.now_ms, Some(JIGGLE));
            }
        } else {
            self.anim.set(Anim::Jiggle, 0.0);
        }
        let now_ms = self.now_ms;
        let jiggle = move |index: usize| {
            if !jiggling || index >= favorites {
                return (0.0, 0.0);
            }
            let phase = now_ms as f32 / 1000.0 * std::f32::consts::TAU * 4.0 + index as f32 * 2.1;
            (1.6 * phase.sin(), 1.2 * (phase * 1.3).cos())
        };

        // The favorites sit on glass, like the tvOS dock; the other rows lie on the wallpaper.
        let (first, last) = (tile_rect(0), tile_rect(columns - 1));
        let platter =
            Rect::new(first.x, first.y - scroll_y, last.right() - first.x, first.h).grow(PLATTER);
        // One pass: the tint and the lit rim are the glass shader's own.
        p.wallpaper_glass_tinted(
            platter,
            PLATTER_RADIUS,
            self.config.wallpaper,
            if self.more_contrast { 70 } else { 196 },
        );

        let visible = p.display.visible_rect();
        p.list.push_clip(visible);
        for (index, item) in items.iter().enumerate() {
            let (mut jx, mut jy) = jiggle(index);
            if focused_id == Some(item.id) && !calm {
                let lean = 5.0
                    * (std::f32::consts::PI * self.focus_progress(item.id).clamp(0.0, 1.0)).sin();
                jx += self.home_sway.0 * lean;
                jy += self.home_sway.1 * lean;
            }
            let rect = tile_rect(index).translate(jx, jy - scroll_y);
            if rect.bottom() < visible.y || rect.y > visible.bottom() {
                continue;
            }
            let progress = self.focus_progress(item.id);
            let mut style = p.focus_style(&THEME.tile, progress);
            if focused_id == Some(item.id) {
                // The banner that takes the focus swells a little past its size and settles,
                // which stands in for the tvOS focus spring.
                if !calm {
                    style.scale += 0.03 * (std::f32::consts::PI * progress.clamp(0.0, 1.0)).sin();
                    style.scale += 0.035 * settled;
                }
                // Held OK: back to the size it has without the focus, as a pressed tvOS icon.
                style.scale -= (style.scale - 1.0) * 0.85 * press;
            }
            style.background = Some(Paint::Solid(item.color));
            // The banner turns away from where the focus came, in perspective, and comes back.
            let turn = if focused_id == Some(item.id) && !calm {
                0.0007 * (std::f32::consts::PI * progress.clamp(0.0, 1.0)).sin()
            } else {
                0.0
            };
            if turn > 0.0 {
                p.list.push_tilt(
                    rect.center(),
                    self.home_sway.0 * turn,
                    self.home_sway.1 * turn,
                );
            }
            if self.tile_has_image(item.package.as_deref(), false) {
                let mut under_style = style;
                under_style.background = None;
                let under = 2.0 * crate::theme::vars::EDGE_SOFTNESS / p.display.scale;
                p.list.draw_box_under(rect, &under_style, under);
            } else {
                p.draw_box(rect, &style);
            }
            self.draw_tile_image(p, item.package.as_deref(), rect, &style, false);
            if focused_id == Some(item.id) && progress > 0.0 && progress < 1.0 && !calm {
                // A band of light crosses the banner as it takes the focus.
                let at = -0.3 + 1.6 * progress;
                let clear = Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 0,
                };
                p.list.fill(
                    rect.scaled_about_center(style.scale),
                    style.border_radius * style.scale,
                    Paint::Gradient(Gradient::new(
                        115.0,
                        &[
                            GradientStop {
                                color: clear,
                                position: at - 0.28,
                            },
                            GradientStop {
                                color: Color { a: 70, ..clear },
                                position: at,
                            },
                            GradientStop {
                                color: clear,
                                position: at + 0.28,
                            },
                        ],
                    )),
                );
            }
            if turn > 0.0 {
                p.list.pop_tilt();
            }
            // Under the Control Center every banner steps back with the picture.
            let under: f32 = if interactive || self.control_closing {
                0.0
            } else {
                90.0
            };
            let dim = if focused_id == Some(item.id) {
                under
            } else {
                under.max(settled * 115.0)
            };
            if dim > 0.0 {
                p.list.fill(
                    rect,
                    style.border_radius,
                    Paint::Solid(Color {
                        r: 4,
                        g: 7,
                        b: 13,
                        a: dim.round() as u8,
                    }),
                );
            }
            if item.package.is_none() {
                p.icon(crate::icons::PLUS, rect, 36.0, Color::WHITE);
            } else {
                self.draw_tile_name(p, item.package.as_deref(), &item.label, rect, &style, false);
            }
            let origin = if item.id >= ids::GRID_BASE {
                Origin::Grid(item.id)
            } else {
                Origin::Dock(index)
            };
            self.anchor_context_menu(origin, rect.scaled_about_center(style.scale));
        }
        // The name appears only under the banner that has the focus, as on tvOS.
        for (index, item) in items.iter().enumerate() {
            // Not at rest: the row is against the bottom edge, with no room under it.
            let progress = self.focus_progress(item.id).clamp(0.0, 1.0) * unfolded;
            if progress <= 0.0 || item.package.is_none() {
                continue;
            }
            let tile = tile_rect(index).translate(0.0, -scroll_y);
            p.text_with(
                &item.label,
                Rect::new(tile.x - 14.0, tile.bottom() + 16.0, tile.w + 28.0, 26.0),
                Align::Center,
                // tvOS Body: 29 pt Medium, which is 19.3 units; 20 lands on whole pixels at 1080p.
                &TextStyle {
                    size: 20.0,
                    weight: 500,
                    letter_spacing: 0.0,
                },
                super::painter::fade(Color::WHITE, progress),
            );
        }
        p.list.pop_clip();

        // The status capsule: the time, and the controls glyph once it has the focus, when it
        // turns white like every focused control of tvOS.
        if !interactive {
            return;
        }
        let capsule_start = p.list.len();
        let focus = self.focus_progress(ids::CONTROL_CENTER).clamp(0.0, 1.0);
        let capsule = STATUS.scaled_about_center(1.0 + 0.06 * focus);
        p.wallpaper_glass(capsule, capsule.h / 2.0, self.config.wallpaper);
        p.list.fill(
            capsule,
            capsule.h / 2.0,
            Paint::Solid(crate::ui::style::mix_color(
                Color {
                    r: 18,
                    g: 25,
                    b: 32,
                    a: if self.more_contrast { 225 } else { 95 },
                },
                Color {
                    r: 246,
                    g: 248,
                    b: 252,
                    a: 240,
                },
                focus,
            )),
        );
        let ink = crate::ui::style::mix_color(
            Color::WHITE,
            Color {
                r: 16,
                g: 22,
                b: 32,
                a: 255,
            },
            focus,
        );
        // The capsule says when the TV is offline (Wi-Fi off), as tvOS shows its status icons
        // beside the time; otherwise it carries the controls glyph.
        let (glyph, glyph_ink) = if self.radios.0 {
            (crate::icons::CONTROL_CENTER, ink)
        } else {
            (
                crate::icons::WIFI,
                Color {
                    r: 255,
                    g: 159,
                    b: 10,
                    a: 255,
                },
            )
        };
        p.icon(
            glyph,
            Rect::new(capsule.x + 10.0, capsule.y, 40.0, capsule.h),
            24.0,
            glyph_ink,
        );
        p.text_with(
            &self.clock.text(),
            Rect::new(capsule.x + 46.0, capsule.y + 8.0, capsule.w - 60.0, 32.0),
            Align::Center,
            &TextStyle {
                size: 22.0,
                weight: 500,
                letter_spacing: 0.0,
            },
            ink,
        );
        // The library has the screen to itself.
        if unfolded > 0.0 {
            p.list.fade_from(capsule_start, 1.0 - unfolded);
        }
    }

    pub(super) fn activate_home(&mut self, id: u32) -> Vec<Effect> {
        match id {
            ids::FAVORITES_ADD => {
                self.open_editor(true);
                vec![]
            }
            id if id >= ids::GRID_BASE => self.activate_apps(id),
            id if id >= ids::FAVORITE_BASE => {
                let package = self
                    .config
                    .favorites
                    .get((id - ids::FAVORITE_BASE) as usize)
                    .cloned();
                package.map(|p| self.open_app(&p)).unwrap_or_default()
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::tests::Rig;
    use crate::app::{Key, Screen};

    #[test]
    fn at_rest_only_the_first_row_shows_against_the_bottom_edge() {
        assert_eq!(tile_rect(0, false), Rect::new(54.0, 561.0, 212.0, 120.0));
        assert_eq!(tile_rect(4, false), Rect::new(1014.0, 561.0, 212.0, 120.0));
        assert_eq!(1280.0 - tile_rect(4, false).right(), tile_rect(0, false).x);
        assert_eq!(1280.0 - tile_rect(5, true).right(), tile_rect(0, true).x);
        for six in [false, true] {
            let columns = if six { 6 } else { 5 };
            // The platter of the first row is whole on the screen; the second row is below it.
            assert!(tile_rect(0, six).bottom() + PLATTER < 720.0);
            assert!(tile_rect(columns, six).y >= 720.0);
        }
    }

    #[test]
    fn down_from_the_first_row_unfolds_the_library_and_up_folds_it_back() {
        let mut rig = Rig::new();
        rig.frame();
        assert_eq!(rig.app.scroll_y, 0.0);
        rig.press(Key::Down);
        rig.frame();
        // The first row is at the top now, and the picture goes out of focus (scrolled).
        assert_eq!(rig.app.scroll_y, TOP - UNFOLDED_TOP);
        rig.press(Key::Up);
        rig.frame();
        assert_eq!(rig.app.scroll_y, 0.0);
    }

    #[test]
    fn up_from_the_first_row_reaches_the_status_capsule_which_opens_the_controls() {
        let mut rig = Rig::new();
        rig.press(Key::Up);
        assert_eq!(rig.focused(), Some(ids::CONTROL_CENTER));
        rig.press(Key::Ok);
        assert_eq!(rig.app.screen(), &Screen::ControlCenter);
    }

    #[test]
    fn home_starts_with_favorites_then_other_installed_apps() {
        let rig = Rig::new();
        let items = rig.app.home_items();
        assert_eq!(items[0].id, ids::FAVORITE_BASE);
        assert_eq!(items[4].id, ids::FAVORITE_BASE + 4);
        assert!(items[5].id >= ids::GRID_BASE);
        assert_eq!(items.len(), rig.app.catalog.apps().len());
    }

    #[test]
    fn remote_moves_across_the_shelf_and_into_the_next_row() {
        let mut rig = Rig::new();
        for _ in 0..4 {
            rig.press(Key::Right);
        }
        assert_eq!(rig.focused(), Some(rig.app.home_items()[4].id));
        for _ in 0..4 {
            rig.press(Key::Left);
        }
        rig.press(Key::Down);
        assert_eq!(rig.focused(), Some(rig.app.home_items()[5].id));
        rig.press(Key::Up);
        assert_eq!(rig.focused(), Some(rig.app.home_items()[0].id));
    }

    #[test]
    fn an_empty_favorites_list_keeps_the_editor_available() {
        let mut rig = Rig::new();
        rig.app.config.favorites.clear();
        rig.frame();
        assert_eq!(rig.app.home_items()[0].id, ids::FAVORITES_ADD);
        rig.app.focus.focus(ids::FAVORITES_ADD);
        rig.press(Key::Ok);
        assert_eq!(rig.app.screen(), &Screen::Editor);
    }

    #[test]
    fn nonfavorite_tiles_open_the_matching_catalog_app() {
        let mut rig = Rig::new();
        let item = rig.app.home_items()[5].package.clone().unwrap();
        rig.app.focus.focus(rig.app.home_items()[5].id);
        let effects = rig.press(Key::Ok);
        assert!(effects.iter().any(|effect| {
            matches!(effect, Effect::OpenApp { package, .. } if package == &item)
        }));
    }
}
