//! The context menu: what the Menu button opens over an app of the grid or a favorite of the dock.
//!
//! It is a state of the app, not a screen: the screen under it stays as it is, with its focus on
//! the item the menu belongs to, and the keys go only to the menu while it is open.

use super::painter::Painter;
use super::{dock, ids, Anim, Effect, Key, LumeApp, PendingFocus, Screen, Tab};
use crate::home::config::MAX_FAVORITES;
use crate::i18n::{Arg, PluralId, StringId};
use crate::keys::{keycode_parts, KeyTarget};
use crate::render::Rect;
use crate::text::Align;
use crate::theme::{StateFlags, Transition, THEME};
use crate::ui::focus::FocusId;

/// Space between the card's edge and its items. With the card's radius (20) it keeps the corners of
/// the focused item (`--radius`, 12) concentric with the card's: 20 = 12 + 8.
const PADDING: f32 = 8.0;
/// Space between two items: none, they are the rows of one list (the focus has no ring to make
/// room for).
const ITEM_GAP: f32 = 0.0;
/// Height of a disabled item, which has its reason under the name.
const NOTE_ITEM_HEIGHT: f32 = 72.0;
/// Only for very short menus: the longest item decides the width, so both sides keep the same margin.
const MIN_WIDTH: f32 = 200.0;
/// Room left after the longest text (see `draw_context_menu`).
const TEXT_SLACK: f32 = 4.0;
/// Space between the card and the name box of the favorite it belongs to, over the dock. The box
/// has room above the letters, so the name is still about 18 below the card.
const DOCK_GAP: f32 = 4.0;
/// Space between the card and the focus ring of the tile it belongs to, over the grid: close enough
/// to read as the tile's. The card's top is the ring's top.
const GRID_GAP: f32 = 8.0;
/// Space between the card of a key-mapping row's menu and the row it belongs to.
const ROW_GAP: f32 = 8.0;
/// Where the name of a disabled item sits in it, and its reason below: the two lines are centered
/// in the 72 of the item (by their ink, measured with scripts/measure-ink.py).
const NOTE_NAME_TOP: f32 = 11.0;
const NOTE_LINE_PITCH: f32 = 28.0;

/// How far below its place the panel starts as it fades in.
const RISE: f32 = 8.0;

/// The fade of the panel as it opens and closes (`--focus-time`).
const FADE: Transition = Transition {
    duration_ms: 150.0,
    curve: crate::theme::Curve::EaseOut,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    AddToHome,
    RemoveFromHome,
    /// Starts moving the favorite along the dock (see `dock_move`).
    Move,
    AppInfo,
    Uninstall,
    /// Key mapping: opens the listening for a new key of the row's target.
    AddKey,
    /// Key mapping: takes the key `code` off the row's target.
    RemoveKey(i32),
    /// Key mapping: puts the row's command back to its factory keys.
    RestoreDefault,
    /// Key mapping: takes the shortcut of the row away, with all its keys.
    RemoveShortcut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuItem {
    pub action: MenuAction,
    /// A disabled item is shown with its reason and never has the focus.
    pub enabled: bool,
}

/// Where the menu was opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// A tile of the Apps grid (its focus id).
    Grid(FocusId),
    /// A favorite of the dock (its place).
    Dock(usize),
    /// A row of the key mapping screen (its focus id): the menu opens under it.
    KeyRow(FocusId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenu {
    pub origin: Origin,
    pub package: String,
    /// What the menu of a key mapping row acts on.
    pub target: Option<KeyTarget>,
    pub items: Vec<MenuItem>,
    /// The item with the focus.
    pub selected: usize,
    /// Closed: it only fades out, and takes no more keys.
    closing: bool,
    /// The item it belongs to, as drawn in the last frame.
    anchor: Option<Rect>,
}

impl ContextMenu {
    pub(super) fn new(origin: Origin, package: String, items: Vec<MenuItem>) -> ContextMenu {
        let selected = items.iter().position(|i| i.enabled).unwrap_or(0);
        ContextMenu {
            origin,
            package,
            target: None,
            items,
            selected,
            closing: false,
            anchor: None,
        }
    }

    /// Moves the focus to the next enabled item in `step` (-1 up, 1 down); it stops at the ends.
    fn step(&mut self, step: isize) {
        let mut index = self.selected as isize;
        loop {
            index += step;
            match self.items.get(index as usize) {
                _ if index < 0 => return,
                None => return,
                Some(item) if item.enabled => {
                    self.selected = index as usize;
                    return;
                }
                Some(_) => {}
            }
        }
    }
}

/// The id whose focus animation an item of the menu uses.
fn item_id(index: usize) -> FocusId {
    ids::MENU_BASE + index as FocusId
}

/// Where the panel goes over the dock: centered above `anchor` (the magnified icon) and above its
/// name bubble, inside `safe`.
pub fn dock_panel(anchor: Rect, size: (f32, f32), safe: Rect) -> Rect {
    let (w, h) = size;
    let x = (anchor.center().0 - w / 2.0).clamp(safe.x, (safe.right() - w).max(safe.x));
    let bottom = anchor.y - dock::LABEL_GAP - dock::LABEL_HEIGHT - DOCK_GAP;
    Rect::new(x, (bottom - h).max(safe.y), w, h)
}

/// Where the panel goes over the grid: right of the tile `anchor`, or left of it when it does not
/// fit, with its top on the tile's top, inside `safe`.
pub fn grid_panel(anchor: Rect, size: (f32, f32), safe: Rect) -> Rect {
    let (w, h) = size;
    let right = anchor.right() + GRID_GAP;
    let x = if right + w <= safe.right() {
        right
    } else {
        (anchor.x - GRID_GAP - w).max(safe.x)
    };
    let y = anchor.y.min(safe.bottom() - h).max(safe.y);
    Rect::new(x, y, w, h)
}

/// Where the panel goes under a row of the key mapping screen: with its right edge on the row's,
/// 8 below it, or 8 above when it does not fit under; moved only as far as it needs to stay inside
/// `safe`.
pub fn row_panel(anchor: Rect, size: (f32, f32), safe: Rect) -> Rect {
    let (w, h) = size;
    let x = (anchor.right() - w).clamp(safe.x, (safe.right() - w).max(safe.x));
    let below = anchor.bottom() + ROW_GAP;
    let y = if below + h <= safe.bottom() {
        below
    } else {
        anchor.y - ROW_GAP - h
    };
    Rect::new(x, y.clamp(safe.y, (safe.bottom() - h).max(safe.y)), w, h)
}

impl LumeApp {
    /// The open menu, if any (for tests).
    #[doc(hidden)]
    pub fn context_menu(&self) -> Option<&ContextMenu> {
        self.context_menu.as_ref().filter(|m| !m.closing)
    }

    /// Whether the open menu takes the keys.
    pub(super) fn menu_is_open(&self) -> bool {
        self.context_menu().is_some()
    }

    /// The Menu button with no menu open: opens the menu of the focused item, where it has one.
    pub(super) fn open_context_menu(&mut self) {
        let Some(id) = self.focus.current_id() else {
            return;
        };
        let menu = match self.screen {
            Screen::Main(Tab::Apps) => self.grid_menu(id),
            Screen::Main(Tab::Home) => {
                if id >= ids::GRID_BASE {
                    self.grid_menu(id)
                } else {
                    self.dock_menu(id)
                }
            }
            _ => None,
        };
        if let Some(menu) = menu {
            self.show_context_menu(menu);
        }
    }

    /// Opens `menu` with its fade, the focus on its first enabled item.
    pub(super) fn show_context_menu(&mut self, menu: ContextMenu) {
        // Every item starts unfocused: the last menu may have left one of these lit.
        for index in 0..menu.items.len() {
            self.anim.set(Anim::Focus(item_id(index)), 0.0);
        }
        self.anim
            .animate_to(Anim::MenuFade, 0.0, 1.0, self.now_ms, Some(FADE));
        self.anim.animate_to(
            Anim::Focus(item_id(menu.selected)),
            0.0,
            1.0,
            self.now_ms,
            self.focus_transition(),
        );
        self.context_menu = Some(menu);
    }

    fn grid_menu(&self, id: FocusId) -> Option<ContextMenu> {
        if id == ids::GRID_SETTINGS {
            return None;
        }
        let app = self
            .catalog
            .apps()
            .get(id.checked_sub(ids::GRID_BASE)? as usize)?;
        let favorite = self.config.favorites.contains(&app.package);
        let mut items = vec![if favorite {
            MenuItem {
                action: MenuAction::RemoveFromHome,
                enabled: true,
            }
        } else {
            MenuItem {
                action: MenuAction::AddToHome,
                enabled: !self.config.favorites_full(),
            }
        }];
        items.push(MenuItem {
            action: MenuAction::AppInfo,
            enabled: true,
        });
        if !app.system {
            items.push(MenuItem {
                action: MenuAction::Uninstall,
                enabled: true,
            });
        }
        Some(ContextMenu::new(
            Origin::Grid(id),
            app.package.clone(),
            items,
        ))
    }

    fn dock_menu(&self, id: FocusId) -> Option<ContextMenu> {
        if !(ids::FAVORITE_BASE..ids::GRID_BASE).contains(&id) {
            return None;
        }
        let index = (id - ids::FAVORITE_BASE) as usize;
        let count = self.config.favorites.len();
        let package = self.config.favorites.get(index)?.clone();
        let item = |action| MenuItem {
            action,
            enabled: true,
        };
        let mut items = Vec::new();
        if count > 1 {
            items.push(item(MenuAction::Move));
        }
        items.push(item(MenuAction::RemoveFromHome));
        items.push(item(MenuAction::AppInfo));
        Some(ContextMenu::new(Origin::Dock(index), package, items))
    }

    /// A key while the menu is open: the focus stays in it.
    pub(super) fn menu_key(&mut self, key: Key) -> Vec<Effect> {
        let Some(menu) = self.context_menu.as_mut() else {
            return vec![];
        };
        let before = menu.selected;
        match key {
            Key::Up => menu.step(-1),
            Key::Down => menu.step(1),
            Key::Left | Key::Right => {}
            Key::Back | Key::Menu | Key::Home => {
                self.close_context_menu();
                return vec![];
            }
            Key::Ok => {
                let action = menu.items.get(menu.selected).filter(|i| i.enabled);
                let action = action.map(|i| i.action);
                let (origin, package) = (menu.origin, menu.package.clone());
                let target = menu.target.clone();
                self.close_context_menu();
                return action
                    .map(|a| match origin {
                        Origin::KeyRow(id) => self.run_key_menu_action(a, id, target),
                        _ => self.run_menu_action(a, origin, &package),
                    })
                    .unwrap_or_default();
            }
        }
        let after = menu.selected;
        if before != after {
            let transition = self.focus_transition();
            let now = self.now_ms;
            self.anim
                .animate_to(Anim::Focus(item_id(before)), 1.0, 0.0, now, transition);
            self.anim
                .animate_to(Anim::Focus(item_id(after)), 0.0, 1.0, now, transition);
        }
        vec![]
    }

    /// Closes the menu with its fade; the focus is still on the item it belongs to.
    pub(super) fn close_context_menu(&mut self) {
        if let Some(menu) = self.context_menu.as_mut().filter(|m| !m.closing) {
            menu.closing = true;
            let visible = self.anim.get(Anim::MenuFade, 1.0, self.now_ms);
            self.anim
                .animate_to(Anim::MenuFade, visible, 0.0, self.now_ms, Some(FADE));
        }
    }

    fn run_menu_action(
        &mut self,
        action: MenuAction,
        origin: Origin,
        package: &str,
    ) -> Vec<Effect> {
        let favorite = self.config.favorites.iter().position(|p| p == package);
        match action {
            MenuAction::AddToHome => {
                if self.config.add_favorite(package) {
                    return vec![Effect::SaveConfig];
                }
            }
            MenuAction::RemoveFromHome => {
                let Some(index) = favorite else { return vec![] };
                let count = self.config.favorites.len() - 1;
                if let Origin::Dock(_) = origin {
                    // The favorite now in its place, or the one before when it was the last.
                    let place = index.min(count.saturating_sub(1));
                    self.begin_dock_transition(Some(place));
                    let id = match count {
                        0 => ids::FAVORITES_ADD,
                        _ => ids::FAVORITE_BASE + place as FocusId,
                    };
                    self.pending_focus = Some(PendingFocus::Id(id));
                }
                self.config.remove_favorite(index);
                return vec![Effect::SaveConfig];
            }
            MenuAction::Move => {
                if favorite.is_some() {
                    self.start_dock_move(package);
                }
            }
            MenuAction::AppInfo => {
                return vec![Effect::OpenAppInfo {
                    package: package.to_string(),
                }]
            }
            MenuAction::Uninstall => {
                return vec![Effect::Uninstall {
                    package: package.to_string(),
                }]
            }
            // The menus of the key mapping screen go through `run_key_menu_action`.
            _ => {}
        }
        vec![]
    }

    /// The catalog changed: a menu whose app is gone closes.
    pub(super) fn check_context_menu(&mut self) {
        let gone = self.context_menu().is_some_and(|m| {
            matches!(m.origin, Origin::Grid(_) | Origin::Dock(_))
                && !self.catalog.contains(&m.package)
        });
        if gone {
            self.close_context_menu();
        }
    }

    /// Records where the item the menu belongs to was drawn in this frame.
    pub(super) fn anchor_context_menu(&mut self, origin: Origin, rect: Rect) {
        if let Some(menu) = self.context_menu.as_mut().filter(|m| m.origin == origin) {
            menu.anchor = Some(rect);
        }
    }

    /// The text of an item.
    fn menu_label(&self, action: MenuAction) -> String {
        let t = self.translator;
        let id = match action {
            MenuAction::AddToHome => StringId::MenuAddHome,
            MenuAction::RemoveFromHome => StringId::MenuRemoveHome,
            MenuAction::Move => StringId::MenuMove,
            MenuAction::AppInfo => StringId::MenuAppInfo,
            MenuAction::Uninstall => StringId::MenuUninstall,
            MenuAction::AddKey => StringId::KeymapAddKey,
            MenuAction::RestoreDefault => StringId::KeymapRestoreAction,
            MenuAction::RemoveShortcut => StringId::KeymapRemoveAction,
            MenuAction::RemoveKey(code) => {
                let key = match keycode_parts(code, &t) {
                    (name, Some(code)) => format!("{name} {code}"),
                    (name, None) => name,
                };
                return t.format(StringId::KeymapRemoveKey, &[Arg::Str(&key)]);
            }
        };
        t.get(id).into_owned()
    }

    /// Why a disabled item is disabled.
    fn menu_note(&self, action: MenuAction) -> String {
        match action {
            MenuAction::RestoreDefault => self
                .translator
                .get(StringId::KeymapAlreadyDefault)
                .into_owned(),
            _ => self.translator.plural(
                PluralId::EditorLimitReached,
                MAX_FAVORITES as i64,
                &[Arg::Int(MAX_FAVORITES as i64)],
            ),
        }
    }

    /// Draws the panel over everything else, fading it as it opens and closes.
    pub(super) fn draw_context_menu(&mut self, p: &mut Painter) {
        let visible = self.anim.get(Anim::MenuFade, 1.0, self.now_ms);
        let Some(menu) = self.context_menu.clone() else {
            return;
        };
        if menu.closing && visible <= 0.0 {
            self.context_menu = None;
            self.anim.set(Anim::MenuFade, 1.0);
            return;
        }
        let Some(anchor) = menu.anchor else { return };
        let item_style = p.style(&THEME.menu_item, StateFlags::NONE);
        let note_style = p.style(&THEME.menu_note, StateFlags::NONE);
        let side = item_style.padding[1] + item_style.padding[3];
        let rows: Vec<(MenuItem, String, Option<String>)> = menu
            .items
            .iter()
            .map(|item| {
                let note = (!item.enabled).then(|| self.menu_note(item.action));
                (*item, self.menu_label(item.action), note)
            })
            .collect();
        let mut inner = MIN_WIDTH - 2.0 * PADDING;
        for (_, label, note) in &rows {
            // A few units of slack: at 1.5x the drawn line can come out a rounding error wider
            // than measured, and it would be ellipsized although the item fits it.
            inner = inner.max(p.measure(label, &item_style) + side + TEXT_SLACK);
            if let Some(note) = note {
                inner = inner.max(p.measure(note, &note_style) + side + TEXT_SLACK);
            }
        }
        let item_height = item_style.height.unwrap_or(56.0);
        let heights: Vec<f32> = rows
            .iter()
            .map(|(_, _, note)| {
                if note.is_some() {
                    NOTE_ITEM_HEIGHT
                } else {
                    item_height
                }
            })
            .collect();
        let width = (inner + 2.0 * PADDING).ceil();
        let height = heights.iter().sum::<f32>()
            + heights.len().saturating_sub(1) as f32 * ITEM_GAP
            + 2.0 * PADDING;
        let safe = p.display.safe_area();
        let panel = match menu.origin {
            Origin::Dock(_) => dock_panel(anchor, (width, height), safe),
            Origin::Grid(_) => grid_panel(anchor, (width, height), safe),
            Origin::KeyRow(_) => row_panel(anchor, (width, height), safe),
        };
        // It rises a little into place as it fades in, and sinks as it fades out.
        let panel = panel.translate(0.0, (1.0 - visible) * RISE);

        let start = p.list.len();
        let panel_style = p.style(&THEME.menu, StateFlags::NONE);
        p.draw_box(panel, &panel_style);
        let mut y = panel.y + PADDING;
        for (index, ((item, label, note), h)) in rows.iter().zip(&heights).enumerate() {
            let rect = Rect::new(panel.x + PADDING, y, inner, *h);
            y += h + ITEM_GAP;
            let style = if item.enabled {
                p.focus_style(&THEME.menu_item, self.focus_progress(item_id(index)))
            } else {
                p.style(&THEME.menu_item, StateFlags::DISABLED)
            };
            p.draw_box(rect, &style);
            let text = Rect::new(rect.x + style.padding[3], rect.y, rect.w - side, rect.h);
            match note {
                None => p.text(label, text, Align::Start, &style),
                Some(note) => {
                    let line = Rect::new(text.x, text.y + NOTE_NAME_TOP, text.w, 32.0);
                    p.text(label, line, Align::Start, &style);
                    let mut note_style = note_style;
                    note_style.opacity = style.opacity;
                    p.text(
                        note,
                        line.translate(0.0, NOTE_LINE_PITCH),
                        Align::Start,
                        &note_style,
                    );
                }
            }
        }
        if visible < 1.0 {
            p.list.fade_from(start, visible);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::Display;

    const SIZE: (f32, f32) = (280.0, 200.0);

    #[test]
    fn the_layout_is_on_the_ruler() {
        super::super::assert_on_ruler(&[
            ("PADDING", PADDING),
            ("ITEM_GAP", ITEM_GAP),
            ("NOTE_ITEM_HEIGHT", NOTE_ITEM_HEIGHT),
            ("MIN_WIDTH", MIN_WIDTH),
            ("DOCK_GAP", DOCK_GAP),
            ("GRID_GAP", GRID_GAP),
            ("NOTE_LINE_PITCH", NOTE_LINE_PITCH),
            ("RISE", RISE),
            ("ROW_GAP", ROW_GAP),
        ]);
    }

    #[test]
    fn over_the_grid_the_panel_is_right_of_the_tile_or_left_in_the_last_column() {
        let safe = Display::new(1280, 720).safe_area();
        let first = Rect::new(64.0, 144.0, 172.0, 96.75);
        let panel = grid_panel(first, SIZE, safe);
        assert_eq!(panel.x, first.right() + GRID_GAP);
        assert_eq!(panel.y, first.y, "aligned with the top of the tile");
        let sixth = Rect::new(1280.0 - 64.0 - 172.0, 144.0, 172.0, 96.75);
        let panel = grid_panel(sixth, SIZE, safe);
        assert_eq!(panel.right(), sixth.x - GRID_GAP, "left of the tile");
        // A tile low on the screen keeps the panel inside the safe area.
        let low = Rect::new(64.0, 600.0, 172.0, 96.75);
        let panel = grid_panel(low, SIZE, safe);
        assert!(panel.bottom() <= safe.bottom() && panel.y >= safe.y);
    }

    #[test]
    fn under_a_row_the_panel_is_right_aligned_and_above_it_when_it_does_not_fit() {
        let safe = Display::new(1280, 720).safe_area();
        let row = Rect::new(96.0, 300.0, 1088.0, 64.0);
        let panel = row_panel(row, SIZE, safe);
        assert_eq!(panel.right(), row.right(), "right edges meet");
        assert_eq!(panel.y, row.bottom() + ROW_GAP, "under the row");
        let low = Rect::new(96.0, 600.0, 1088.0, 64.0);
        let panel = row_panel(low, SIZE, safe);
        assert_eq!(panel.bottom(), low.y - ROW_GAP, "above the row");
        // A list taller than both sides leaves room for is moved just enough to fit.
        let tall = row_panel(Rect::new(96.0, 300.0, 1088.0, 64.0), (280.0, 576.0), safe);
        assert!(tall.y >= safe.y && tall.bottom() <= safe.bottom());
    }

    #[test]
    fn over_the_dock_the_panel_is_above_the_name_bubble_inside_the_safe_area() {
        let safe = Display::new(1280, 720).safe_area();
        let icon = Rect::new(600.0, 560.0, 102.4, 102.4);
        let panel = dock_panel(icon, SIZE, safe);
        assert!(
            (panel.center().0 - icon.center().0).abs() < 1e-3,
            "centered"
        );
        assert!(panel.bottom() < icon.y - dock::LABEL_GAP - dock::LABEL_HEIGHT);
        let edge = Rect::new(safe.x, 560.0, 102.4, 102.4);
        let panel = dock_panel(edge, SIZE, safe);
        assert_eq!(panel.x, safe.x, "moved sideways to stay inside");
    }
}
