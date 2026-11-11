//! Home, laid out like tvOS: the wallpaper full-bleed, a first row of favorites on a glass
//! platter low on the screen, and the rest of the apps in rows of five below it. The focused
//! banner grows over a soft shadow and shows its name under it.

use super::context_menu::Origin;
use super::dock::DockItem;
use super::painter::Painter;
use super::topbar::{self, BarLayout, TopBar};
use super::{ids, Effect, LumeApp, Tab};
use crate::i18n::StringId;
use crate::render::Rect;
use crate::text::{Align, TextStyle};
use crate::theme::{Color, Gradient, GradientStop, Paint, StateFlags, THEME};
use crate::ui::focus::{Focusable, Row};

/// tvOS's five-column grid (320 pt columns, 40 pt gutters, 80 pt side margins on a 1920 pt
/// screen) on the 1280-unit canvas, with 16:9 banners, which is what Android TV apps ship.
const COLUMNS: usize = 5;
const LEFT: f32 = 54.0;
const TOP: f32 = 496.0;
pub(super) const WIDTH: f32 = 212.0;
const HEIGHT: f32 = 120.0;
const PITCH_X: f32 = 240.0;
/// The next row peeks from the bottom edge, which is what says there is more below.
const PITCH_Y: f32 = 184.0;
/// How far the glass platter of the first row reaches around its banners.
const PLATTER: f32 = 20.0;

fn tile_rect(index: usize) -> Rect {
    Rect::new(
        LEFT + (index % COLUMNS) as f32 * PITCH_X,
        TOP + (index / COLUMNS) as f32 * PITCH_Y,
        WIDTH,
        HEIGHT,
    )
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
        let tabs = BarLayout::new(
            p,
            &self.translator.get(StringId::TabHome),
            &self.translator.get(StringId::TabApps),
            &self.clock,
        );
        let mut rows = vec![topbar::row(&tabs, Tab::Home, self.inputs_available)];
        for (line_index, line) in items.chunks(COLUMNS).enumerate() {
            let mut row = Row::new(
                line.iter()
                    .enumerate()
                    .map(|(col, item)| {
                        Focusable::new(item.id, tile_rect(line_index * COLUMNS + col))
                    })
                    .collect(),
            );
            if line_index == 0 {
                row = row.with_entry_down(line[0].id);
            }
            rows.push(row);
        }
        self.update_focus(rows);

        let focused_id = self.focus.current_id();
        let focus_rect =
            focused_id.and_then(|id| items.iter().position(|item| item.id == id).map(tile_rect));
        let bottom = p.display.visible_rect().bottom();
        let content_bottom = tile_rect(items.len().saturating_sub(1)).bottom() + 48.0;
        self.reveal_vertical(focus_rect, content_bottom, TOP - 40.0, bottom - TOP + 40.0);
        if focused_id.is_some_and(|id| id < ids::FAVORITES_ADD) {
            self.scroll_to(0.0);
        }
        let scroll_y = self.current_scroll_y();

        // A light veil low on the screen, so the row that peeks reads over a bright beach.
        let scrim = Paint::Gradient(Gradient::new(
            180.0,
            &[
                GradientStop {
                    color: Color::TRANSPARENT,
                    position: 0.0,
                },
                GradientStop {
                    color: Color {
                        r: 4,
                        g: 7,
                        b: 13,
                        a: 130,
                    },
                    position: 1.0,
                },
            ],
        ));
        p.list
            .fill(Rect::new(0.0, 400.0, 1280.0, 320.0), 0.0, scrim);

        // The favorites sit on glass, like the tvOS dock; the other rows lie on the wallpaper.
        let platter = Rect::new(
            LEFT,
            TOP - scroll_y,
            (COLUMNS - 1) as f32 * PITCH_X + WIDTH,
            HEIGHT,
        )
        .grow(PLATTER);
        p.wallpaper_glass(platter, 30.0, self.config.wallpaper);
        p.list.fill(
            platter,
            30.0,
            Paint::Solid(Color {
                r: 12,
                g: 16,
                b: 24,
                a: 70,
            }),
        );
        p.list.ring(
            platter,
            30.0,
            1.0,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 70,
            },
        );

        let visible = p.display.visible_rect();
        p.list.push_clip(visible);
        for (index, item) in items.iter().enumerate() {
            let rect = tile_rect(index).translate(0.0, -scroll_y);
            if rect.bottom() < visible.y || rect.y > visible.bottom() {
                continue;
            }
            let progress = self.focus_progress(item.id);
            let mut style = p.focus_style(&THEME.tile, progress);
            if focused_id == Some(item.id) {
                // The banner that takes the focus swells a little past its size and settles,
                // which stands in for the tvOS focus spring.
                style.scale += 0.03 * (std::f32::consts::PI * progress.clamp(0.0, 1.0)).sin();
            }
            style.background = Some(Paint::Solid(item.color));
            if self.tile_has_image(item.package.as_deref(), false) {
                let mut under_style = style;
                under_style.background = None;
                let under = 2.0 * crate::theme::vars::EDGE_SOFTNESS / p.display.scale;
                p.list.draw_box_under(rect, &under_style, under);
            } else {
                p.draw_box(rect, &style);
            }
            self.draw_tile_image(p, item.package.as_deref(), rect, &style, false);
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
            let progress = self.focus_progress(item.id).clamp(0.0, 1.0);
            if progress <= 0.0 || item.package.is_none() {
                continue;
            }
            let tile = tile_rect(index).translate(0.0, -scroll_y);
            p.text_with(
                &item.label,
                Rect::new(tile.x - 14.0, tile.bottom() + 16.0, tile.w + 28.0, 26.0),
                Align::Center,
                &TextStyle {
                    size: 18.0,
                    weight: 700,
                    letter_spacing: 0.0,
                },
                super::painter::fade(Color::WHITE, progress),
            );
        }
        p.list.pop_clip();

        // Full controls appear when navigated to; otherwise the hero keeps the small time capsule.
        if focused_id.is_some_and(|id| id <= ids::CONTROL_CENTER) {
            self.fade_tab_content(p);
            topbar::draw_glass_bar(p, self.config.wallpaper, false);
            let progress = |id| self.focus_progress(id);
            TopBar {
                tabs: &tabs,
                active: Tab::Home,
                home_label: &self.translator.get(StringId::TabHome),
                apps_label: &self.translator.get(StringId::TabApps),
                clock: &self.clock,
                inputs: self.inputs_available,
                focus: &progress,
            }
            .draw(p);
        } else {
            let capsule = Rect::new(1104.0, 24.0, 112.0, 48.0);
            p.wallpaper_glass(capsule, 24.0, self.config.wallpaper);
            p.list.fill(
                capsule,
                24.0,
                Paint::Solid(Color {
                    r: 18,
                    g: 25,
                    b: 32,
                    a: 95,
                }),
            );
            p.text_with(
                &self.clock.text(),
                Rect::new(1112.0, 32.0, 96.0, 32.0),
                Align::Center,
                &TextStyle {
                    size: 22.0,
                    weight: 700,
                    letter_spacing: 0.0,
                },
                Color::WHITE,
            );
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
    fn five_banners_fill_the_first_row_inside_equal_margins() {
        assert_eq!(tile_rect(0), Rect::new(54.0, 496.0, 212.0, 120.0));
        assert_eq!(tile_rect(4), Rect::new(1014.0, 496.0, 212.0, 120.0));
        assert_eq!(tile_rect(5), Rect::new(54.0, 680.0, 212.0, 120.0));
        assert_eq!(1280.0 - tile_rect(4).right(), tile_rect(0).x);
        // The second row peeks from the bottom of the screen.
        assert!(tile_rect(5).y < 720.0);
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
