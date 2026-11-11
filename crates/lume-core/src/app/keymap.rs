//! The remote control key mapping screen, and the two screens where an app or a settings shortcut is
//! chosen to get a key.
//!
//! The screen is a column of rows with a vertical focus only: one row per command and per shortcut,
//! the "add" rows and the factory reset button. What can be done
//! with a row (add or remove a key, restore, remove the shortcut) is in the context menu that OK
//! opens, so no row has small targets inside it. Listening for a key and the factory reset
//! confirmation are light cards over the screen, like the context menu, without a dark layer.

use super::context_menu::{ContextMenu, MenuAction, MenuItem, Origin};
use super::editor::{wrap, ButtonKind};
use super::page::{self, CONTENT_TOP, LEFT, WIDTH};
use super::painter::Painter;
use super::settings::ITEM_KEY_MAPPING;
use super::{apps, Effect, LumeApp, PendingFocus, Screen};
use crate::i18n::{Arg, StringId};
use crate::icons;
use crate::keys::{keycode_parts, CommandTarget, Key, KeyTarget, SettingShortcut};
use crate::render::Rect;
use crate::text::{Align, TextStyle};
use crate::theme::{vars, Paint, StateFlags, THEME};
use crate::ui::focus::{FocusId, Focusable, Row};

/// The command rows: this plus the place in `CommandTarget::ALL`.
pub const CMD_BASE: FocusId = 6010;
/// The rows of the settings shortcuts that have a key: this plus the place in the keymap's list.
pub const SETTING_BASE: FocusId = 6100;
pub const ADD_SETTING: FocusId = 6200;
/// The rows of the app shortcuts: this plus the place in the keymap's list.
pub const APP_BASE: FocusId = 6300;
pub const ADD_APP: FocusId = 6400;
pub const RESTORE_FACTORY: FocusId = 6500;
pub const CONFIRM_RESTORE: FocusId = 7100;
pub const CONFIRM_CANCEL: FocusId = 7101;
/// The tiles of the "Choose an app" screen: this plus the place among the apps without a shortcut.
pub const PICK_BASE: FocusId = 8000;
/// The rows of the "Choose a shortcut" screen: this plus the place among the shortcuts without a key.
pub const PICK_SETTING_BASE: FocusId = 8500;

const PANEL_GAP: f32 = 24.0;
const PANEL_PADDING_X: f32 = 32.0;
/// The heading's distance from the panel's top, and its height: the box is centered between the
/// panel's top and the first row (12 above it, 12 below), so the text has the same space on both sides.
const HEADING_TOP: f32 = 12.0;
const HEADING_HEIGHT: f32 = 32.0;
/// Where the first row starts in a panel, the height of a row and the distance between two.
const FIRST_ROW: f32 = 56.0;
const ROW_HEIGHT: f32 = 64.0;
const ROW_PITCH: f32 = 72.0;
/// What a panel has besides its rows: 56 at the top, 24 at the bottom, minus the gap that the last
/// row does not need (a panel of `n` rows is `72 + 72 n` tall).
const PANEL_CHROME: f32 = 72.0;
/// Space inside a row, before the label and after the keycaps.
const ROW_PADDING: f32 = 24.0;
/// The keycap is centered in the row: (64 - 40) / 2 above and below.
const KEYCAP_TOP: f32 = 12.0;
const KEYCAP_HEIGHT: f32 = 40.0;
const KEYCAP_GAP: f32 = 16.0;
/// Room left after a measured text, so it is not ellipsized by a rounding error.
const TEXT_SLACK: f32 = 4.0;
/// Space between the label of a row and its first keycap.
const LABEL_GAP: f32 = 16.0;
const BUTTON_HEIGHT: f32 = 48.0;
const BUTTON_MIN_WIDTH: f32 = 160.0;
const BOTTOM_PADDING: f32 = 96.0;
/// The glyph of the "add" rows and the gap before its text.
const ADD_GLYPH: f32 = 24.0;
const ADD_GAP: f32 = 8.0;

/// The light cards. Listening: 560x188 at (360, 264). Confirmation: 560x260 at (360, 232).
const CARD_X: f32 = 360.0;
const CARD_WIDTH: f32 = 560.0;
const CARD_PADDING: f32 = 32.0;
const LISTEN_Y: f32 = 264.0;
const LISTEN_HEIGHT: f32 = 188.0;
const CONFIRM_Y: f32 = 232.0;
const CONFIRM_HEIGHT: f32 = 260.0;
/// Inside a card: the title (28/800), the second line and the strip of the hint or the warning.
const TITLE_TOP: f32 = 32.0;
const TITLE_HEIGHT: f32 = 44.0;
const LINE_TOP: f32 = 76.0;
const LINE_HEIGHT: f32 = 40.0;
const STRIP_TOP: f32 = 116.0;
const BUTTONS_TOP: f32 = 180.0;
const CARD_BUTTON_GAP: f32 = 16.0;
const CARD_BUTTON_MIN_WIDTH: f32 = 140.0;
const CARD_BUTTON_PADDING: f32 = 56.0;

#[derive(Clone, Debug, Default)]
pub struct KeyMapState {
    /// When listening for a remote keypress, contains the target being configured.
    pub listening: Option<KeyTarget>,
    /// Whether a reserved key was rejected while listening.
    pub reserved_warning: bool,
    /// Whether the factory restore confirmation dialog is open.
    pub confirm_restore: bool,
}

/// What a line of the screen is.
#[derive(Clone, Debug, PartialEq)]
enum LineKind {
    /// A command, a settings shortcut or an app shortcut, with its keys.
    Mapping(KeyTarget),
    AddSetting,
    AddApp,
}

struct Line {
    id: FocusId,
    /// In the page, before the scrolling.
    rect: Rect,
    kind: LineKind,
}

struct PanelBox {
    rect: Rect,
    heading: StringId,
}

/// Where everything is on the screen, in the page (before the scrolling).
struct Layout {
    panels: Vec<PanelBox>,
    lines: Vec<Line>,
    restore: Rect,
    restore_label: String,
    content_height: f32,
}

impl Layout {
    fn rect_of(&self, id: FocusId) -> Option<Rect> {
        match id {
            RESTORE_FACTORY => Some(self.restore),
            _ => self.lines.iter().find(|l| l.id == id).map(|l| l.rect),
        }
    }
}

/// A panel with the given rows: its box, and the rectangles of its rows.
fn panel(top: f32, heading: StringId, rows: Vec<(FocusId, LineKind)>) -> (PanelBox, Vec<Line>) {
    let rect = Rect::new(
        LEFT,
        top,
        WIDTH,
        PANEL_CHROME + ROW_PITCH * rows.len() as f32,
    );
    let lines = rows
        .into_iter()
        .enumerate()
        .map(|(index, (id, kind))| Line {
            id,
            kind,
            rect: Rect::new(
                LEFT + PANEL_PADDING_X,
                top + FIRST_ROW + index as f32 * ROW_PITCH,
                WIDTH - 2.0 * PANEL_PADDING_X,
                ROW_HEIGHT,
            ),
        })
        .collect();
    (PanelBox { rect, heading }, lines)
}

impl LumeApp {
    /// Returns true when the key listening modal is actively capturing raw keycodes.
    pub fn is_listening_key(&self) -> bool {
        self.screen == Screen::KeyMap && self.keymap.listening.is_some()
    }

    pub(super) fn open_keymap(&mut self) {
        self.screen = Screen::KeyMap;
        self.keymap = KeyMapState::default();
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::FirstContent);
        self.focus.set_rows(vec![]);
    }

    pub(super) fn back_from_keymap(&mut self) -> Vec<Effect> {
        if self.keymap.listening.is_some() {
            self.stop_listening();
            return vec![];
        }
        if self.keymap.confirm_restore {
            self.keymap.confirm_restore = false;
            self.pending_focus = Some(PendingFocus::Id(RESTORE_FACTORY));
            return vec![];
        }
        self.screen = Screen::Settings;
        self.scroll_to_top();
        self.focus.set_rows(vec![]);
        self.pending_focus = Some(PendingFocus::Id(ITEM_KEY_MAPPING));
        vec![]
    }

    /// Back on the "Choose an app" or "Choose a shortcut" screen: the mapping screen, on the "add"
    /// row it was opened from.
    pub(super) fn back_from_picker(&mut self, add_row: FocusId) -> Vec<Effect> {
        self.screen = Screen::KeyMap;
        self.scroll_to_top();
        self.focus.set_rows(vec![]);
        self.pending_focus = Some(PendingFocus::Id(add_row));
        vec![]
    }

    // ---- listening ---------------------------------------------------------------------------

    fn start_listening(&mut self, target: KeyTarget) {
        self.keymap.listening = Some(target);
        self.keymap.reserved_warning = false;
    }

    /// Ends the listening and puts the focus on the row it was started from (the "add" row when
    /// the shortcut was not created).
    fn stop_listening(&mut self) {
        if let Some(target) = self.keymap.listening.take() {
            self.pending_focus = Some(PendingFocus::Id(self.row_id_for(&target)));
        }
        self.keymap.reserved_warning = false;
    }

    /// A logical key while listening (the desktop viewer, which has no key codes): Back cancels,
    /// and the keys the interface needs are refused.
    pub(super) fn listening_key(&mut self, key: Key) -> Vec<Effect> {
        match key {
            Key::Back => self.stop_listening(),
            Key::Menu => {}
            _ => self.keymap.reserved_warning = true,
        }
        vec![]
    }

    /// A key code while listening: Back cancels, a reserved key (arrows, OK, Home, power) is
    /// refused with a warning and never recorded, any other key is given to the target.
    pub(super) fn listening_code(&mut self, code: i32) -> Vec<Effect> {
        if crate::keys::is_back_code(code) {
            self.stop_listening();
            return vec![];
        }
        let target = self.keymap.listening.clone();
        match target {
            Some(target) if !crate::keys::is_reserved_code(code) => {
                if !self.keys.add_code(&target, code) {
                    self.keymap.reserved_warning = true;
                    return vec![];
                }
                let message = self.translator.get(StringId::ToastKeyMapped).into_owned();
                self.show_toast(message);
                self.stop_listening();
                vec![Effect::SaveKeys]
            }
            _ => {
                self.keymap.reserved_warning = true;
                vec![]
            }
        }
    }

    // ---- rows --------------------------------------------------------------------------------

    /// The focus id of the row of `target`: its own while it has one, otherwise the "add" row
    /// that lists it.
    fn row_id_for(&self, target: &KeyTarget) -> FocusId {
        match target {
            KeyTarget::Command(c) => {
                CMD_BASE + CommandTarget::ALL.iter().position(|x| x == c).unwrap_or(0) as FocusId
            }
            KeyTarget::Setting(s) => self
                .keys
                .setting_shortcuts()
                .iter()
                .position(|(x, _)| x == s)
                .map_or(ADD_SETTING, |i| SETTING_BASE + i as FocusId),
            KeyTarget::App(pkg) => self
                .keys
                .app_shortcuts()
                .iter()
                .position(|(x, _)| x == pkg)
                .map_or(ADD_APP, |i| APP_BASE + i as FocusId),
        }
    }

    /// The place of the row of `target` among the shortcuts of its panel, if it has a row there.
    fn shortcut_index(&self, target: &KeyTarget) -> Option<usize> {
        match target {
            KeyTarget::Command(_) => None,
            KeyTarget::Setting(s) => self
                .keys
                .setting_shortcuts()
                .iter()
                .position(|(x, _)| x == s),
            KeyTarget::App(pkg) => self.keys.app_shortcuts().iter().position(|(x, _)| x == pkg),
        }
    }

    /// Where the focus goes after keys were taken off `target`: its row while it has one, the row
    /// that took its place when it was removed, or the "add" row when it was the last.
    fn focus_after_removal(&self, target: &KeyTarget, before: Option<usize>) -> FocusId {
        if self.shortcut_index(target).is_some() {
            return self.row_id_for(target);
        }
        match (target, before) {
            (KeyTarget::Setting(_), Some(i)) => {
                if i < self.keys.setting_shortcuts().len() {
                    SETTING_BASE + i as FocusId
                } else {
                    ADD_SETTING
                }
            }
            (KeyTarget::App(_), Some(i)) => {
                if i < self.keys.app_shortcuts().len() {
                    APP_BASE + i as FocusId
                } else {
                    ADD_APP
                }
            }
            _ => self.row_id_for(target),
        }
    }

    /// The target a row of keys belongs to.
    fn target_of_row(&self, id: FocusId) -> Option<KeyTarget> {
        if (CMD_BASE..SETTING_BASE).contains(&id) {
            let command = CommandTarget::ALL.get((id - CMD_BASE) as usize)?;
            return Some(KeyTarget::Command(*command));
        }
        if (SETTING_BASE..ADD_SETTING).contains(&id) {
            let (setting, _) = self
                .keys
                .setting_shortcuts()
                .get((id - SETTING_BASE) as usize)?;
            return Some(KeyTarget::Setting(*setting));
        }
        if (APP_BASE..ADD_APP).contains(&id) {
            let (package, _) = self.keys.app_shortcuts().get((id - APP_BASE) as usize)?;
            return Some(KeyTarget::App(package.clone()));
        }
        None
    }

    /// The settings shortcuts that still have no key.
    fn unmapped_settings(&self) -> Vec<SettingShortcut> {
        SettingShortcut::ALL
            .into_iter()
            .filter(|&s| !self.keys.has_setting(s))
            .collect()
    }

    /// The apps that still have no shortcut, in the order of the Apps grid.
    fn unmapped_apps(&self) -> Vec<&crate::catalog::App> {
        self.catalog
            .apps()
            .iter()
            .filter(|app| !self.keys.has_app(&app.package))
            .collect()
    }

    // ---- activation --------------------------------------------------------------------------

    pub(super) fn activate_keymap(&mut self, id: FocusId) -> Vec<Effect> {
        if self.keymap.listening.is_some() {
            return vec![];
        }
        if self.keymap.confirm_restore {
            match id {
                CONFIRM_CANCEL => {
                    self.keymap.confirm_restore = false;
                    self.pending_focus = Some(PendingFocus::Id(RESTORE_FACTORY));
                }
                CONFIRM_RESTORE => {
                    self.keys.reset();
                    self.keymap.confirm_restore = false;
                    self.pending_focus = Some(PendingFocus::Id(RESTORE_FACTORY));
                    return vec![Effect::SaveKeys];
                }
                _ => {}
            }
            return vec![];
        }
        match id {
            RESTORE_FACTORY => {
                self.keymap.confirm_restore = true;
                self.pending_focus = Some(PendingFocus::Id(CONFIRM_CANCEL));
                vec![]
            }
            ADD_SETTING => {
                self.open_pick_setting();
                vec![]
            }
            ADD_APP => {
                self.open_pick_app();
                vec![]
            }
            _ => {
                if let Some(target) = self.target_of_row(id) {
                    if self.keys.codes_for(&target).is_empty() {
                        // Nothing to choose from: straight to the listening.
                        self.start_listening(target);
                    } else {
                        self.open_row_menu(id, target);
                    }
                }
                vec![]
            }
        }
    }

    /// The menu of a row: add a key, take each key off, restore the command or remove the shortcut.
    fn open_row_menu(&mut self, id: FocusId, target: KeyTarget) {
        let item = |action| MenuItem {
            action,
            enabled: true,
        };
        let mut items = vec![item(MenuAction::AddKey)];
        items.extend(
            self.keys
                .codes_for(&target)
                .iter()
                .map(|&code| item(MenuAction::RemoveKey(code))),
        );
        items.push(match target {
            KeyTarget::Command(_) => MenuItem {
                action: MenuAction::RestoreDefault,
                enabled: !self.keys.is_target_default(&target),
            },
            _ => item(MenuAction::RemoveShortcut),
        });
        let mut menu = ContextMenu::new(Origin::KeyRow(id), String::new(), items);
        menu.target = Some(target);
        self.show_context_menu(menu);
    }

    /// An item of the menu of a row was chosen.
    pub(super) fn run_key_menu_action(
        &mut self,
        action: MenuAction,
        _row: FocusId,
        target: Option<KeyTarget>,
    ) -> Vec<Effect> {
        match action {
            MenuAction::AddKey => {
                if let Some(target) = target {
                    self.start_listening(target);
                }
                vec![]
            }
            MenuAction::RemoveKey(code) => {
                let Some(target) = target else { return vec![] };
                let before = self.shortcut_index(&target);
                self.keys.remove_code(&target, code);
                self.pending_focus =
                    Some(PendingFocus::Id(self.focus_after_removal(&target, before)));
                vec![Effect::SaveKeys]
            }
            MenuAction::RestoreDefault | MenuAction::RemoveShortcut => {
                let Some(target) = target else { return vec![] };
                let before = self.shortcut_index(&target);
                self.keys.reset_target(&target);
                self.pending_focus =
                    Some(PendingFocus::Id(self.focus_after_removal(&target, before)));
                vec![Effect::SaveKeys]
            }
            _ => vec![],
        }
    }

    // ---- layout ------------------------------------------------------------------------------

    fn keymap_layout(&self, p: &mut Painter) -> Layout {
        let mut panels = Vec::new();
        let mut lines = Vec::new();
        let mut top = CONTENT_TOP;
        let mut add = |heading, rows| {
            let (panel_box, mut panel_lines) = panel(top, heading, rows);
            top = panel_box.rect.bottom() + PANEL_GAP;
            panels.push(panel_box);
            lines.append(&mut panel_lines);
        };

        add(
            StringId::KeymapSectionCommands,
            CommandTarget::ALL
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    (
                        CMD_BASE + i as FocusId,
                        LineKind::Mapping(KeyTarget::Command(*c)),
                    )
                })
                .collect(),
        );

        let mut rows: Vec<_> = self
            .keys
            .setting_shortcuts()
            .iter()
            .enumerate()
            .map(|(i, (s, _))| {
                (
                    SETTING_BASE + i as FocusId,
                    LineKind::Mapping(KeyTarget::Setting(*s)),
                )
            })
            .collect();
        if !self.unmapped_settings().is_empty() {
            rows.push((ADD_SETTING, LineKind::AddSetting));
        }
        add(StringId::KeymapSectionSettings, rows);

        let mut rows: Vec<_> = self
            .keys
            .app_shortcuts()
            .iter()
            .enumerate()
            .map(|(i, (pkg, _))| {
                (
                    APP_BASE + i as FocusId,
                    LineKind::Mapping(KeyTarget::App(pkg.clone())),
                )
            })
            .collect();
        if !self.unmapped_apps().is_empty() {
            rows.push((ADD_APP, LineKind::AddApp));
        }
        add(StringId::KeymapSectionApps, rows);

        let restore_label = self
            .translator
            .get(StringId::KeymapRestoreDefaults)
            .into_owned();
        let button_style = p.style(&THEME.button_secondary, StateFlags::NONE);
        let width = (p.measure(&restore_label, &button_style)
            + button_style.padding[1]
            + button_style.padding[3])
            .max(BUTTON_MIN_WIDTH);
        let restore = Rect::new(LEFT, top, width.ceil(), BUTTON_HEIGHT);
        Layout {
            panels,
            lines,
            restore,
            restore_label,
            content_height: restore.bottom() + BOTTOM_PADDING,
        }
    }

    // ---- drawing -----------------------------------------------------------------------------

    pub(super) fn draw_keymap(&mut self, p: &mut Painter) {
        let layout = self.keymap_layout(p);
        let t = self.translator;

        // The focus: nothing while listening (Back is the only key it takes), the two buttons of
        // the confirmation, otherwise one row per line.
        let mut dialog_rects = None;
        if self.keymap.listening.is_some() {
            self.update_focus(vec![]);
        } else if self.keymap.confirm_restore {
            let (restore, cancel) = self.confirm_buttons(p);
            dialog_rects = Some((restore, cancel));
            self.update_focus(vec![Row::new(vec![
                Focusable::new(CONFIRM_RESTORE, restore),
                Focusable::new(CONFIRM_CANCEL, cancel),
            ])]);
        } else {
            let mut rows: Vec<Row> = layout
                .lines
                .iter()
                .map(|l| Row::new(vec![Focusable::new(l.id, l.rect)]))
                .collect();
            rows.push(Row::new(vec![Focusable::new(
                RESTORE_FACTORY,
                layout.restore,
            )]));
            self.update_focus(rows);
        }

        let focused = self.focus.current_id();
        let view_height = p.display.visible_rect().bottom().max(720.0);
        let rect = focused.and_then(|id| layout.rect_of(id));
        self.reveal_vertical(rect, layout.content_height, 0.0, view_height);
        if focused == Some(CMD_BASE) {
            // The first row brings the header back with it.
            self.scroll_to(0.0);
        }
        let scroll_y = self.current_scroll_y();

        page::draw_header(
            p,
            &[
                &t.get(StringId::SettingsTitle),
                &t.get(StringId::SettingsKeyMapping),
            ],
            &t.get(StringId::KeymapTitle),
            Some(&t.get(StringId::KeymapSubtitle)),
            scroll_y,
        );

        let visible = |rect: Rect| rect.bottom() >= -32.0 && rect.y <= view_height + 32.0;

        let panel_style = p.style(&THEME.panel, StateFlags::NONE);
        let heading_style = p.style(&THEME.card_heading, StateFlags::NONE);
        for panel_box in &layout.panels {
            let rect = panel_box.rect.translate(0.0, -scroll_y);
            if !visible(rect) {
                continue;
            }
            p.draw_box(rect, &panel_style);
            let heading = Rect::new(
                rect.x + PANEL_PADDING_X,
                rect.y + HEADING_TOP,
                rect.w - 2.0 * PANEL_PADDING_X,
                HEADING_HEIGHT,
            );
            let text = t.get(panel_box.heading).to_uppercase();
            p.text_with(
                &text,
                heading,
                Align::Start,
                &Painter::text_style(&heading_style),
                vars::PRIMARY,
            );
        }

        for line in &layout.lines {
            let rect = line.rect.translate(0.0, -scroll_y);
            if matches!(line.kind, LineKind::Mapping(_)) {
                self.anchor_context_menu(Origin::KeyRow(line.id), rect);
            }
            if !visible(rect) {
                continue;
            }
            match &line.kind {
                LineKind::Mapping(target) => self.draw_mapping_row(p, line.id, rect, target),
                LineKind::AddSetting => {
                    let label = t.get(StringId::KeymapAddSetting).into_owned();
                    self.draw_add_row(p, line.id, rect, &label);
                }
                LineKind::AddApp => {
                    let label = t.get(StringId::KeymapAddApp).into_owned();
                    self.draw_add_row(p, line.id, rect, &label);
                }
            }
        }

        let restore = layout.restore.translate(0.0, -scroll_y);
        if visible(restore) {
            self.draw_button(
                p,
                restore,
                &layout.restore_label,
                RESTORE_FACTORY,
                ButtonKind::Secondary,
            );
        }

        if let Some((restore, cancel)) = dialog_rects {
            self.draw_confirm_dialog(p, restore, cancel);
        } else if self.keymap.listening.is_some() {
            self.draw_listening_card(p);
        }
    }

    /// The name of a target as the user knows it.
    fn target_name(&self, target: &KeyTarget) -> String {
        let t = self.translator;
        match target {
            KeyTarget::Command(c) => t.get(c.title_id()).into_owned(),
            KeyTarget::Setting(s) => t.get(s.title_id()).into_owned(),
            KeyTarget::App(pkg) => self
                .catalog
                .get(pkg)
                .map_or_else(|| pkg.clone(), |app| app.label.clone()),
        }
    }

    /// A row: the name at the left, the keys as keycaps at the right, the last ending 24 inside.
    fn draw_mapping_row(&self, p: &mut Painter, id: FocusId, rect: Rect, target: &KeyTarget) {
        let t = self.translator;
        let style = p.focus_style(&THEME.editor_row, self.focus_progress(id));
        p.draw_box(rect, &style);

        let keycap = p.style(&THEME.keycap, StateFlags::NONE);
        let caption = p.style(&THEME.about_license, StateFlags::NONE);
        let side = keycap.padding[1] + keycap.padding[3];
        // Measured, never estimated: (name, code, name width, code width, keycap width).
        let caps: Vec<_> = self
            .keys
            .codes_for(target)
            .iter()
            .map(|&code| {
                let (name, number) = keycode_parts(code, &t);
                let name_width = p.measure(&name, &keycap).ceil();
                let number_width = number
                    .as_ref()
                    .map_or(0.0, |n| p.measure(n, &caption).ceil());
                let gap = if number.is_some() { keycap.gap } else { 0.0 };
                let width = side + name_width + gap + number_width;
                (name, number, name_width, number_width, width)
            })
            .collect();

        let mut x = rect.right() - ROW_PADDING;
        let mut keys_width = 0.0;
        for cap in caps.iter().rev() {
            x -= cap.4;
            keys_width += cap.4 + KEYCAP_GAP;
            let cap_rect = Rect::new(x, rect.y + KEYCAP_TOP, cap.4, KEYCAP_HEIGHT);
            p.draw_box(cap_rect, &keycap);
            let name = Rect::new(
                cap_rect.x + keycap.padding[3],
                cap_rect.y,
                cap.2 + TEXT_SLACK,
                cap_rect.h,
            );
            p.text(&cap.0, name, Align::Start, &keycap);
            if let Some(number) = &cap.1 {
                let number_rect = Rect::new(
                    name.x + cap.2 + keycap.gap,
                    cap_rect.y,
                    cap.3 + TEXT_SLACK,
                    cap_rect.h,
                );
                p.text(number, number_rect, Align::Start, &caption);
            }
            x -= KEYCAP_GAP;
        }
        let label_width = rect.w
            - 2.0 * ROW_PADDING
            - if keys_width > 0.0 {
                keys_width - KEYCAP_GAP + LABEL_GAP
            } else {
                0.0
            };
        let label = Rect::new(rect.x + ROW_PADDING, rect.y, label_width, rect.h);
        p.text(&self.target_name(target), label, Align::Start, &style);
    }

    /// The row that adds a shortcut: an outline and a plus until it has the focus, then a row.
    fn draw_add_row(&self, p: &mut Painter, id: FocusId, rect: Rect, label: &str) {
        let progress = self.focus_progress(id);
        let mut style = p.focus_style(&THEME.editor_row, progress);
        style.background = Some(Paint::Solid(super::painter::fade(
            vars::SURFACE_FOCUS,
            progress,
        )));
        p.list.ring(
            rect,
            style.border_radius,
            2.0,
            super::painter::fade(vars::BORDER, 1.0 - progress),
        );
        p.draw_box(rect, &style);
        let text_style = p.style(&THEME.chip, StateFlags::NONE);
        let glyph = Rect::new(rect.x + ROW_PADDING, rect.y, ADD_GLYPH, rect.h);
        p.icon(icons::PLUS, glyph, ADD_GLYPH, text_style.color);
        let text = Rect::new(
            glyph.right() + ADD_GAP,
            rect.y,
            rect.w - 2.0 * ROW_PADDING - ADD_GLYPH - ADD_GAP,
            rect.h,
        );
        p.text(label, text, Align::Start, &text_style);
    }

    /// The light card the overlays are drawn on: the context menu's, without the dark layer.
    fn draw_light_card(&self, p: &mut Painter, card: Rect) {
        let style = p.style(&THEME.menu, StateFlags::NONE);
        p.draw_box(card, &style);
    }

    /// The listening card: no button and no focus, Back cancels.
    fn draw_listening_card(&self, p: &mut Painter) {
        let t = self.translator;
        let card = Rect::new(CARD_X, LISTEN_Y, CARD_WIDTH, LISTEN_HEIGHT);
        self.draw_light_card(p, card);
        let inner = |top: f32, height: f32| {
            Rect::new(
                card.x + CARD_PADDING,
                card.y + top,
                card.w - 2.0 * CARD_PADDING,
                height,
            )
        };
        p.text_with(
            &t.get(StringId::KeymapListeningTitle),
            inner(TITLE_TOP, TITLE_HEIGHT),
            Align::Start,
            &TextStyle::new(28.0, 800),
            vars::MENU_TEXT,
        );
        let target = self
            .keymap
            .listening
            .as_ref()
            .map(|target| self.target_name(target))
            .unwrap_or_default();
        p.text_with(
            &t.format(StringId::KeymapListeningFor, &[Arg::Str(&target)]),
            inner(LINE_TOP, LINE_HEIGHT),
            Align::Start,
            &TextStyle::new(18.0, 500),
            vars::MENU_TEXT_MUTED,
        );
        // The strip keeps its place: the hint is replaced by the warning, the card does not grow.
        if self.keymap.reserved_warning {
            p.text_with(
                &t.get(StringId::KeymapReservedKeyWarning),
                inner(STRIP_TOP, LINE_HEIGHT),
                Align::Start,
                &TextStyle::new(18.0, 700),
                vars::MENU_DANGER,
            );
        } else {
            p.text_with(
                &t.get(StringId::KeymapListeningHint),
                inner(STRIP_TOP, LINE_HEIGHT),
                Align::Start,
                &TextStyle::new(18.0, 500),
                vars::MENU_TEXT_MUTED,
            );
        }
    }

    /// The two buttons of the confirmation, right-aligned: "Restore" then "Cancel".
    fn confirm_buttons(&self, p: &mut Painter) -> (Rect, Rect) {
        let t = self.translator;
        let style = p.style(&THEME.menu_button, StateFlags::NONE);
        let width = |p: &mut Painter, label: &str| {
            (p.measure(label, &style) + CARD_BUTTON_PADDING)
                .max(CARD_BUTTON_MIN_WIDTH)
                .ceil()
        };
        let restore_w = width(p, &t.get(StringId::EditorRestoreConfirm));
        let cancel_w = width(p, &t.get(StringId::EditorCancel));
        let y = CONFIRM_Y + BUTTONS_TOP;
        let right = CARD_X + CARD_WIDTH - CARD_PADDING;
        let cancel = Rect::new(right - cancel_w, y, cancel_w, BUTTON_HEIGHT);
        let restore = Rect::new(
            cancel.x - CARD_BUTTON_GAP - restore_w,
            y,
            restore_w,
            BUTTON_HEIGHT,
        );
        (restore, cancel)
    }

    /// "Restore the factory mapping?": a light card, the focus on "Cancel" at first.
    fn draw_confirm_dialog(&self, p: &mut Painter, restore: Rect, cancel: Rect) {
        let t = self.translator;
        let card = Rect::new(CARD_X, CONFIRM_Y, CARD_WIDTH, CONFIRM_HEIGHT);
        self.draw_light_card(p, card);
        let inner = |top: f32, height: f32| {
            Rect::new(
                card.x + CARD_PADDING,
                card.y + top,
                card.w - 2.0 * CARD_PADDING,
                height,
            )
        };
        p.text_with(
            &t.get(StringId::KeymapDialogRestoreTitle),
            inner(TITLE_TOP, TITLE_HEIGHT),
            Align::Start,
            &TextStyle::new(28.0, 800),
            vars::MENU_TEXT,
        );
        let body_style = TextStyle::new(18.0, 500);
        let body = t.get(StringId::KeymapDialogRestoreBody).into_owned();
        for (index, line) in wrap(p, &body, &body_style, card.w - 2.0 * CARD_PADDING)
            .iter()
            .take(2)
            .enumerate()
        {
            p.text_with(
                line,
                inner(LINE_TOP + index as f32 * LINE_HEIGHT, LINE_HEIGHT),
                Align::Start,
                &body_style,
                vars::MENU_TEXT_MUTED,
            );
        }
        for (rect, label, id, component) in [
            (
                restore,
                t.get(StringId::EditorRestoreConfirm),
                CONFIRM_RESTORE,
                &THEME.menu_button_danger,
            ),
            (
                cancel,
                t.get(StringId::EditorCancel),
                CONFIRM_CANCEL,
                &THEME.menu_button,
            ),
        ] {
            let style = p.focus_style(component, self.focus_progress(id));
            p.draw_box(rect, &style);
            p.text(&label, rect, Align::Center, &style);
        }
    }

    // ---- choose a shortcut -------------------------------------------------------------------

    pub(super) fn open_pick_setting(&mut self) {
        self.screen = Screen::KeyMapPickSetting;
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::FirstContent);
        self.focus.set_rows(vec![]);
    }

    /// The rows of the "Choose a shortcut" list: the shortcuts without a key, one under the other.
    pub(super) fn pick_setting_rows(&self) -> Vec<(FocusId, SettingShortcut, Rect)> {
        self.unmapped_settings()
            .into_iter()
            .enumerate()
            .map(|(index, setting)| {
                (
                    PICK_SETTING_BASE + index as FocusId,
                    setting,
                    Rect::new(
                        LEFT,
                        CONTENT_TOP + index as f32 * ROW_PITCH,
                        WIDTH,
                        ROW_HEIGHT,
                    ),
                )
            })
            .collect()
    }

    /// OK on a row: the mapping screen again, listening for the key of that shortcut. The shortcut
    /// only exists once a key is recorded.
    pub(super) fn activate_pick_setting(&mut self, id: FocusId) -> Vec<Effect> {
        let setting = self
            .pick_setting_rows()
            .into_iter()
            .find(|(row, _, _)| *row == id)
            .map(|(_, setting, _)| setting);
        if let Some(setting) = setting {
            self.screen = Screen::KeyMap;
            self.scroll_to_top();
            self.start_listening(KeyTarget::Setting(setting));
        }
        vec![]
    }

    pub(super) fn draw_pick_setting(&mut self, p: &mut Painter) {
        let t = self.translator;
        let rows = self.pick_setting_rows();
        self.update_focus(
            rows.iter()
                .map(|(id, _, rect)| Row::new(vec![Focusable::new(*id, *rect)]))
                .collect(),
        );

        let focused = self.focus.current_id();
        let rect = focused.and_then(|id| rows.iter().find(|r| r.0 == id).map(|r| r.2));
        let content_height = rows.last().map_or(CONTENT_TOP, |r| r.2.bottom()) + BOTTOM_PADDING;
        let view_height = p.display.visible_rect().bottom().max(720.0);
        self.reveal_vertical(rect, content_height, 0.0, view_height);
        if rect.is_some_and(|r| r.y < CONTENT_TOP + ROW_PITCH) {
            // The first rows bring the header back with them.
            self.scroll_to(0.0);
        }
        let scroll_y = self.current_scroll_y();

        page::draw_header(
            p,
            &[
                &t.get(StringId::SettingsTitle),
                &t.get(StringId::SettingsKeyMapping),
                &t.get(StringId::KeymapPickSettingTitle),
            ],
            &t.get(StringId::KeymapPickSettingTitle),
            Some(&t.get(StringId::KeymapPickSettingSubtitle)),
            scroll_y,
        );
        for (id, setting, rect) in &rows {
            let rect = rect.translate(0.0, -scroll_y);
            if rect.bottom() < -32.0 || rect.y > view_height + 32.0 {
                continue;
            }
            let style = p.focus_style(&THEME.editor_row, self.focus_progress(*id));
            p.draw_box(rect, &style);
            let label = Rect::new(
                rect.x + ROW_PADDING,
                rect.y,
                rect.w - 2.0 * ROW_PADDING,
                rect.h,
            );
            p.text(&t.get(setting.title_id()), label, Align::Start, &style);
        }
    }

    // ---- choose an app -----------------------------------------------------------------------

    pub(super) fn open_pick_app(&mut self) {
        self.screen = Screen::KeyMapPickApp;
        self.scroll_to_top();
        self.pending_focus = Some(PendingFocus::FirstContent);
        self.focus.set_rows(vec![]);
    }

    /// The cells of the "Choose an app" grid: the apps without a shortcut, six to a line.
    pub(super) fn pick_cells(&self) -> Vec<PickCell> {
        self.unmapped_apps()
            .into_iter()
            .enumerate()
            .map(|(index, app)| {
                let (row, column) = (index / apps::COLUMNS, index % apps::COLUMNS);
                PickCell {
                    id: PICK_BASE + index as FocusId,
                    package: app.package.clone(),
                    label: app.label.clone(),
                    color: app.color,
                    rect: Rect::new(
                        LEFT + column as f32 * (apps::TILE_WIDTH + apps::GAP),
                        CONTENT_TOP + row as f32 * apps::ROW_PITCH,
                        apps::TILE_WIDTH,
                        apps::TILE_HEIGHT,
                    ),
                }
            })
            .collect()
    }

    /// OK on a tile: the mapping screen again, listening for the key of that app.
    pub(super) fn activate_pick_app(&mut self, id: FocusId) -> Vec<Effect> {
        let package = self
            .pick_cells()
            .into_iter()
            .find(|cell| cell.id == id)
            .map(|cell| cell.package);
        if let Some(package) = package {
            self.screen = Screen::KeyMap;
            self.scroll_to_top();
            self.start_listening(KeyTarget::App(package));
        }
        vec![]
    }

    pub(super) fn draw_pick_app(&mut self, p: &mut Painter) {
        let t = self.translator;
        let cells = self.pick_cells();
        let rows: Vec<Row> = cells
            .chunks(apps::COLUMNS)
            .map(|line| Row::new(line.iter().map(|c| Focusable::new(c.id, c.rect)).collect()))
            .collect();
        self.update_focus(rows);

        let focused = self.focus.current_id();
        let rect = focused.and_then(|id| cells.iter().find(|c| c.id == id).map(|c| c.rect));
        let content_height = cells
            .last()
            .map_or(CONTENT_TOP, |c| c.rect.bottom() + 40.0 + 40.0);
        let view_height = p.display.visible_rect().bottom().max(720.0);
        // The name under the focused tile has to be visible too.
        let reveal = rect.map(|r| Rect::new(r.x, r.y, r.w, r.h + 40.0));
        self.reveal_vertical(reveal, content_height, 0.0, view_height);
        if rect.is_some_and(|r| r.y < CONTENT_TOP + apps::ROW_PITCH) {
            // The first line brings the header back with it.
            self.scroll_to(0.0);
        }
        let scroll_y = self.current_scroll_y();

        page::draw_header(
            p,
            &[
                &t.get(StringId::SettingsTitle),
                &t.get(StringId::SettingsKeyMapping),
                &t.get(StringId::KeymapPickAppTitle),
            ],
            &t.get(StringId::KeymapPickAppTitle),
            Some(&t.get(StringId::KeymapPickAppSubtitle)),
            scroll_y,
        );

        let tiles: Vec<_> = cells
            .iter()
            .map(|cell| {
                let rect = cell.rect.translate(0.0, -scroll_y);
                let mut style = p.focus_style(&THEME.tile, self.focus_progress(cell.id));
                style.background = Some(Paint::Solid(cell.color));
                (cell, rect, style)
            })
            .filter(|(_, rect, _)| rect.bottom() + 40.0 >= -32.0 && rect.y <= view_height + 32.0)
            .collect();
        for (cell, rect, style) in &tiles {
            if self.tile_has_image(Some(&cell.package), false) {
                let mut style = *style;
                style.background = None;
                let under = 2.0 * vars::EDGE_SOFTNESS / p.display.scale;
                p.list.draw_box_under(*rect, &style, under);
            } else {
                p.draw_box(*rect, style);
            }
        }
        for (cell, rect, style) in &tiles {
            self.draw_tile_image(p, Some(&cell.package), *rect, style, false);
        }
        for (cell, rect, style) in &tiles {
            self.draw_tile_name(p, Some(&cell.package), &cell.label, *rect, style, false);
            let label_style = p.focus_style(&THEME.tile_label, self.focus_progress(cell.id));
            p.text(
                &cell.label,
                Rect::new(rect.x, rect.bottom() + 8.0, rect.w, 24.0),
                Align::Center,
                &label_style,
            );
        }
    }
}

pub(super) struct PickCell {
    pub(super) id: FocusId,
    pub(super) package: String,
    label: String,
    color: crate::theme::Color,
    rect: Rect,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_is_on_the_ruler() {
        super::super::assert_on_ruler(&[
            ("PANEL_GAP", PANEL_GAP),
            ("PANEL_PADDING_X", PANEL_PADDING_X),
            ("HEADING_TOP", HEADING_TOP),
            ("HEADING_HEIGHT", HEADING_HEIGHT),
            ("FIRST_ROW", FIRST_ROW),
            ("ROW_HEIGHT", ROW_HEIGHT),
            ("ROW_PITCH", ROW_PITCH),
            ("PANEL_CHROME", PANEL_CHROME),
            ("ROW_PADDING", ROW_PADDING),
            ("KEYCAP_TOP", KEYCAP_TOP),
            ("KEYCAP_HEIGHT", KEYCAP_HEIGHT),
            ("KEYCAP_GAP", KEYCAP_GAP),
            ("LABEL_GAP", LABEL_GAP),
            ("BUTTON_HEIGHT", BUTTON_HEIGHT),
            ("BUTTON_MIN_WIDTH", BUTTON_MIN_WIDTH),
            ("BOTTOM_PADDING", BOTTOM_PADDING),
            ("ADD_GLYPH", ADD_GLYPH),
            ("ADD_GAP", ADD_GAP),
            ("CARD_X", CARD_X),
            ("CARD_WIDTH", CARD_WIDTH),
            ("CARD_PADDING", CARD_PADDING),
            ("LISTEN_Y", LISTEN_Y),
            ("LISTEN_HEIGHT", LISTEN_HEIGHT),
            ("CONFIRM_Y", CONFIRM_Y),
            ("CONFIRM_HEIGHT", CONFIRM_HEIGHT),
            ("TITLE_TOP", TITLE_TOP),
            ("TITLE_HEIGHT", TITLE_HEIGHT),
            ("LINE_TOP", LINE_TOP),
            ("LINE_HEIGHT", LINE_HEIGHT),
            ("STRIP_TOP", STRIP_TOP),
            ("BUTTONS_TOP", BUTTONS_TOP),
            ("CARD_BUTTON_GAP", CARD_BUTTON_GAP),
            ("CARD_BUTTON_MIN_WIDTH", CARD_BUTTON_MIN_WIDTH),
            ("CARD_BUTTON_PADDING", CARD_BUTTON_PADDING),
        ]);
    }

    #[test]
    fn a_panel_of_n_rows_is_72_plus_72_n_tall() {
        for n in 0..=11usize {
            let rows = (0..n)
                .map(|i| (SETTING_BASE + i as FocusId, LineKind::AddSetting))
                .collect();
            let (panel_box, lines) = panel(280.0, StringId::KeymapSectionCommands, rows);
            assert_eq!(panel_box.rect.h, 72.0 + 72.0 * n as f32);
            if let Some(first) = lines.first() {
                assert_eq!((first.rect.x, first.rect.y), (96.0, 336.0));
                assert_eq!((first.rect.w, first.rect.h), (1088.0, 64.0));
            }
            if let Some(last) = lines.last() {
                // 24 of padding under the last row.
                assert_eq!(panel_box.rect.bottom() - last.rect.bottom(), 24.0);
            }
        }
    }

    #[test]
    fn the_cards_are_centered_in_the_canvas_within_one_step_of_the_ruler() {
        assert_eq!(CARD_X * 2.0 + CARD_WIDTH, 1280.0);
        for (y, height) in [(LISTEN_Y, LISTEN_HEIGHT), (CONFIRM_Y, CONFIRM_HEIGHT)] {
            assert!((720.0 - (2.0 * y + height)).abs() <= 4.0);
        }
    }
}
