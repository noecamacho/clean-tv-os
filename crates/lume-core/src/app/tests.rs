//! Behavior of the launcher as a whole, driven by keys and frames like the platform shell does.

use super::*;
use crate::catalog::Discovered;
use crate::display::Display;
use crate::render::DisplayList;
use crate::text::TextEngine;

const NETFLIX: &str = "com.netflix.ninja";
const YOUTUBE: &str = "com.google.android.youtube.tv";
const PRIME: &str = "com.amazon.amazonvideo.livingroom";
const SPOTIFY: &str = "com.spotify.tv.android";
const DISNEY: &str = "com.disney.disneyplus";
const NOW: u64 = 1_792_446_300;

pub(super) fn discovered(package: &str, label: &str) -> Discovered {
    Discovered {
        package: package.to_string(),
        label: label.to_string(),
        activity: format!("{package}.Main"),
        leanback: true,
        system: false,
        icon: None,
    }
}

fn catalog(apps: &[(&str, &str)]) -> Catalog {
    let entries: Vec<Discovered> = apps.iter().map(|(p, l)| discovered(p, l)).collect();
    Catalog::from_discovered(&entries, "app.lume.tv")
}

fn standard_catalog() -> Catalog {
    catalog(&[
        (NETFLIX, "Netflix"),
        (YOUTUBE, "YouTube"),
        (PRIME, "Prime Video"),
        (SPOTIFY, "Spotify"),
        (DISNEY, "Disney+"),
        ("com.demo.chrome", "Chrome"),
        ("com.demo.games", "Jogos"),
    ])
}

pub(super) struct Rig {
    pub app: LumeApp,
    text: TextEngine,
    display: Display,
    pub now: u64,
}

impl Rig {
    pub fn new() -> Rig {
        Rig::with_catalog(standard_catalog())
    }

    pub fn with_catalog(catalog: Catalog) -> Rig {
        let mut usage = UsageHistory::new();
        usage.mark_baseline(catalog.apps().iter().map(|a| a.package.as_str()));
        let config = HomeConfig::first_use(&catalog);
        let clock = WallClock {
            unix_secs: NOW,
            utc_offset_secs: 0,
            is_24h: true,
        };
        let display = Display::new(1280, 720);
        let mut rig = Rig {
            app: LumeApp::new(catalog, usage, config, Translator::new(&["pt-BR"]), clock),
            text: TextEngine::new(display.scale),
            display,
            now: 0,
        };
        rig.frame();
        rig
    }

    /// Draws one frame and returns the display list.
    pub fn frame(&mut self) -> DisplayList {
        let mut list = DisplayList::new();
        for _ in 0..4 {
            list.clear();
            self.text.begin_frame();
            let mut painter = Painter {
                list: &mut list,
                text: &mut self.text,
                display: self.display,
            };
            self.app.draw(&mut painter, self.now);
            if !self.text.end_frame() {
                break;
            }
        }
        list
    }

    /// Presses a key, advances 100 ms and draws.
    pub fn press(&mut self, key: Key) -> Vec<Effect> {
        let effects = self.app.handle_key(key, self.now);
        self.now += 100;
        self.frame();
        effects
    }

    pub fn press_all(&mut self, keys: &[Key]) {
        for key in keys {
            self.press(*key);
        }
    }

    /// Moves the clock forward by `ms`, drawing a frame at the end.
    pub fn advance(&mut self, ms: u64) {
        self.now += ms;
        self.frame();
    }

    pub fn focused(&self) -> Option<FocusId> {
        self.app.focused()
    }
}

#[test]
fn the_app_starts_on_the_first_favorite() {
    let rig = Rig::new();
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE));
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
}

#[test]
fn moving_the_focus_animates_the_old_and_new_elements() {
    let mut rig = Rig::new();
    rig.advance(1000);
    assert_eq!(
        rig.app.focus_progress(ids::FAVORITE_BASE),
        1.0,
        "the first element is fully focused"
    );
    rig.app.handle_key(Key::Right, rig.now);
    rig.advance(10);
    rig.advance(60);
    let (old, new) = (
        rig.app.focus_progress(ids::FAVORITE_BASE),
        rig.app.focus_progress(ids::FAVORITE_BASE + 1),
    );
    assert!(
        old < 1.0 && old > 0.0 && new > 0.0 && new < 1.0,
        "mid-animation: {old} {new}"
    );
    rig.advance(1000);
    assert_eq!(rig.app.focus_progress(ids::FAVORITE_BASE), 0.0);
    assert_eq!(rig.app.focus_progress(ids::FAVORITE_BASE + 1), 1.0);
    assert!(
        rig.app.next_deadline(rig.now).is_some(),
        "the clock still ticks"
    );
}

fn open_apps(rig: &mut Rig) {
    // From the favorites: up to the top bar, right to "Apps", OK.
    rig.press_all(&[Key::Up]);
    assert_eq!(rig.focused(), Some(ids::TAB_HOME));
    rig.press_all(&[Key::Right, Key::Ok]);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
}

#[test]
fn up_from_the_first_row_goes_to_the_active_tab_and_right_then_ok_opens_apps() {
    let mut rig = Rig::new();
    rig.press(Key::Up);
    assert_eq!(rig.focused(), Some(ids::TAB_HOME), "the active tab");
    rig.press(Key::Right);
    assert_eq!(rig.focused(), Some(ids::TAB_APPS));
    rig.press(Key::Ok);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(
        rig.focused(),
        Some(ids::TAB_APPS),
        "the focus stays on the tab"
    );
    // Down enters the grid.
    rig.press(Key::Down);
    assert_eq!(rig.focused(), Some(ids::GRID_BASE));
}

#[test]
fn back_on_the_home_root_scrolls_to_the_top_without_closing() {
    let mut rig = Rig::new();
    rig.press(Key::Right);
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE + 1));
    let effects = rig.press(Key::Back);
    assert!(effects.is_empty(), "lume stays open");
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(
        rig.focused(),
        Some(ids::FAVORITE_BASE),
        "the focus returns to the first favorite"
    );
    assert_eq!(rig.app.scroll_y, 0.0);
    // A second Back changes nothing.
    assert!(rig.press(Key::Back).is_empty());
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE));
}

#[test]
fn back_in_the_apps_tab_returns_to_home() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.press(Key::Back);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE));
}

#[test]
fn the_home_button_returns_to_the_start_of_the_home_tab() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press_all(&[Key::Down, Key::Right, Key::Right]);
    rig.press(Key::Home);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(
        rig.focused(),
        Some(ids::FAVORITE_BASE),
        "the first element of the content"
    );
    // Pressing Home again on the home tab keeps everything where it is.
    rig.press_all(&[Key::Down, Key::Right]);
    rig.press(Key::Home);
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE));
}

#[test]
fn the_apps_grid_is_alphabetical_with_home_among_the_apps() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    let labels: Vec<&str> = rig
        .app
        .catalog
        .apps()
        .iter()
        .map(|a| a.label.as_str())
        .collect();
    assert_eq!(
        labels,
        [
            "Chrome",
            "Disney+",
            "Jogos",
            "Netflix",
            "Prime Video",
            "Spotify",
            "YouTube"
        ]
    );
    let rows = rig.app.focus.rows();
    let ids_in_order: Vec<u32> = rows
        .iter()
        .skip(1)
        .flat_map(|r| r.items.iter().map(|i| i.id))
        .collect();
    assert_eq!(ids_in_order.len(), 8, "the seven apps and HOME");
    assert_eq!(
        ids_in_order[2],
        ids::GRID_SETTINGS,
        "HOME comes between Disney+ and Jogos"
    );
    let cells = rows
        .iter()
        .skip(1)
        .map(|r| r.items.len())
        .collect::<Vec<_>>();
    assert_eq!(cells, [6, 2], "6 columns");
}

#[test]
fn the_grid_does_not_wrap_at_the_end_of_a_row() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.press_all(&[Key::Right; 5]);
    // Chrome, Disney+, Jogos, Lume, Netflix, Prime Video: the 6th is the 5th app (index 4).
    assert_eq!(rig.focused(), Some(ids::GRID_BASE + 4));
    rig.press(Key::Right);
    assert_eq!(
        rig.focused(),
        Some(ids::GRID_BASE + 4),
        "stays on the 6th column"
    );
}

#[test]
fn down_into_a_short_last_row_lands_on_its_last_item() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.press_all(&[Key::Right; 4]);
    // Column 5 of the first row is Netflix (index 3, as lume sits before it).
    assert_eq!(rig.focused(), Some(ids::GRID_BASE + 3));
    rig.press(Key::Down);
    // The last row has Spotify and YouTube: the nearest to column 5 is the last one.
    assert_eq!(rig.focused(), Some(ids::GRID_BASE + 6));
}

#[test]
fn up_from_the_first_grid_row_goes_to_the_top_bar() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press_all(&[Key::Down, Key::Right, Key::Right, Key::Up]);
    assert_eq!(rig.focused(), Some(ids::TAB_APPS), "the active tab");
}

#[test]
fn ok_on_the_home_tile_opens_home_settings() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.press_all(&[Key::Right, Key::Right]);
    assert_eq!(rig.focused(), Some(ids::GRID_SETTINGS));
    assert_eq!(rig.press(Key::Ok), []);
    assert_eq!(rig.app.screen(), &Screen::Settings);
}

#[test]
fn the_clock_asks_for_a_frame_when_the_minute_turns() {
    let mut rig = Rig::new();
    rig.advance(1000);
    let deadline = rig.app.next_deadline(rig.now).expect("a deadline");
    assert!(deadline > rig.now && deadline <= rig.now + 60_000);
    // 21:45:00 -> the minute turns after exactly 60 seconds from the clock's own start.
    assert_eq!(rig.app.clock.millis_to_next_minute(), 60_000);
}

#[test]
fn an_idle_app_without_a_visible_clock_change_only_wakes_for_the_clock() {
    let mut rig = Rig::new();
    rig.press(Key::Down);
    rig.advance(5000);
    let deadline = rig.app.next_deadline(rig.now).unwrap();
    assert!(deadline <= rig.now + 60_000);
}

#[test]
fn removing_an_installed_app_updates_the_screens() {
    let mut rig = Rig::new();
    rig.app.catalog.apply(&[], &[YOUTUBE.to_string()]);
    rig.app.config.remove_uninstalled(&rig.app.catalog);
    rig.advance(100);
    assert_eq!(rig.app.config.favorites, [NETFLIX, PRIME, SPOTIFY, DISNEY]);
}

// ---- settings and the home screen editor ---------------------------------------------------

use super::editor::{
    ADD_APP_BASE, CONFIRM_CANCEL, CONFIRM_RESTORE, FAVORITE_BUTTON_BASE, RESTORE, ROUND_FAVORITES,
    WALLPAPER_BASE,
};

fn open_settings(rig: &mut Rig) {
    // The Apps tab, then the "Lume" tile.
    open_apps(rig);
    rig.press(Key::Down);
    press_on(rig, ids::GRID_SETTINGS);
    assert_eq!(rig.app.screen(), &Screen::Settings);
}

fn open_editor(rig: &mut Rig) {
    open_settings(rig);
    assert_eq!(
        rig.focused(),
        Some(ids::SETTINGS_BASE),
        "the first item has the focus"
    );
    rig.press(Key::Ok);
    assert_eq!(rig.app.screen(), &Screen::Editor);
}

/// Puts the focus on `id` (navigation itself is tested separately) and presses OK.
fn press_on(rig: &mut Rig, id: FocusId) -> Vec<Effect> {
    focus_on(rig, id);
    rig.press(Key::Ok)
}

fn focus_on(rig: &mut Rig, id: FocusId) {
    assert!(rig.app.focus.focus(id), "{id} cannot take the focus");
    rig.advance(0);
}

#[test]
fn the_lume_settings_start_on_the_first_item_and_list_four() {
    let mut rig = Rig::new();
    open_settings(&mut rig);
    assert_eq!(rig.focused(), Some(ids::SETTINGS_BASE));
    rig.press(Key::Down);
    assert_eq!(rig.focused(), Some(ids::SETTINGS_BASE + 1));
    rig.press(Key::Down);
    assert_eq!(rig.focused(), Some(ids::SETTINGS_BASE + 2));
    rig.press(Key::Down);
    assert_eq!(rig.focused(), Some(ids::SETTINGS_BASE + 3));
    assert!(
        !rig.app.focus.move_focus(Direction::Down),
        "four items only"
    );
}

#[test]
fn the_keymap_opens_from_the_settings_and_back_returns_to_its_item() {
    let mut rig = Rig::new();
    open_settings(&mut rig);
    rig.press_all(&[Key::Down, Key::Ok]);
    assert_eq!(rig.app.screen(), &Screen::KeyMap);
    rig.press(Key::Back);
    assert_eq!(rig.app.screen(), &Screen::Settings);
    assert_eq!(rig.focused(), Some(ids::SETTINGS_BASE + 1));
}

fn open_key_map(rig: &mut Rig) {
    open_settings(rig);
    rig.press_all(&[Key::Down, Key::Ok]);
    assert_eq!(rig.app.screen(), &Screen::KeyMap);
}

fn open_tv_info(rig: &mut Rig) {
    open_settings(rig);
    rig.press_all(&[Key::Down, Key::Down, Key::Ok]);
    assert_eq!(rig.app.screen(), &Screen::TvInfo);
}

#[test]
fn the_tv_information_waits_for_the_platform_and_is_read_each_time_it_opens() {
    let mut rig = Rig::new();
    open_tv_info(&mut rig);
    assert!(rig.app.wants_device_info(), "asked on opening");
    rig.app
        .apply_device_info(crate::device_info::DeviceInfo::default());
    assert!(!rig.app.wants_device_info(), "read once while it stays");
    rig.press(Key::Back);
    assert_eq!(rig.app.screen(), &Screen::Settings);
    assert!(!rig.app.wants_device_info());
    rig.press(Key::Ok);
    assert!(rig.app.wants_device_info(), "read again on the next visit");
    rig.app.went_to_background();
    assert!(!rig.app.wants_device_info(), "not in the background");
}

#[test]
fn the_tv_information_scrolls_within_the_sheet_and_back_returns_to_its_item() {
    let mut rig = Rig::new();
    open_tv_info(&mut rig);
    rig.app
        .apply_device_info(crate::device_info::DeviceInfo::default());
    rig.advance(0);
    assert!(
        rig.app.tv_max_scroll > 0.0,
        "the sheet is taller than the screen"
    );
    rig.press(Key::Up);
    assert_eq!(rig.app.scroll_y, 0.0, "no scrolling above the top");
    rig.press(Key::Down);
    assert!(rig.app.scroll_y > 0.0);
    for _ in 0..50 {
        rig.press(Key::Down);
    }
    assert_eq!(
        rig.app.scroll_y, rig.app.tv_max_scroll,
        "no scrolling past the end"
    );
    rig.advance(1000);
    rig.press(Key::Back);
    assert_eq!(rig.app.screen(), &Screen::Settings);
    assert_eq!(rig.focused(), Some(ids::SETTINGS_BASE + 2));
    assert_eq!(rig.app.scroll_y, 0.0);
}

#[test]
fn back_from_the_settings_returns_to_the_lume_tile() {
    let mut rig = Rig::new();
    open_settings(&mut rig);
    rig.press(Key::Back);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(rig.focused(), Some(ids::GRID_SETTINGS));
}

#[test]
fn the_gear_opens_the_android_settings() {
    let mut rig = Rig::new();
    rig.press_all(&[Key::Up, Key::Right, Key::Right, Key::Right]);
    assert_eq!(rig.focused(), Some(ids::GEAR));
    assert_eq!(rig.press(Key::Ok), [Effect::OpenAndroidSettings]);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
}

#[test]
fn the_inputs_button_sits_left_of_control_center_and_opens_the_inputs() {
    let mut rig = Rig::new();
    rig.app.inputs_available = true;
    rig.press_all(&[Key::Up, Key::Right, Key::Right]);
    assert_eq!(rig.focused(), Some(ids::INPUTS));
    assert_eq!(rig.press(Key::Ok), [Effect::OpenInputs]);
    rig.press(Key::Right);
    assert_eq!(rig.focused(), Some(ids::CONTROL_CENTER));
    rig.press(Key::Left);
    assert_eq!(rig.focused(), Some(ids::INPUTS));
}

#[test]
fn without_an_inputs_panel_the_button_is_not_there() {
    let mut rig = Rig::new();
    rig.app.inputs_available = false;
    rig.press_all(&[Key::Up, Key::Right, Key::Right, Key::Right]);
    assert_eq!(rig.focused(), Some(ids::GEAR));
    rig.advance(1000);
    assert!(!rig.app.focus.focus(ids::INPUTS));
}

#[test]
fn the_about_screen_scrolls_and_goes_back_to_the_settings() {
    let mut rig = Rig::new();
    open_settings(&mut rig);
    rig.press_all(&[Key::Down, Key::Down, Key::Down, Key::Ok]);
    assert_eq!(rig.app.screen(), &Screen::About);
    assert_eq!(rig.app.about_scroll, 0.0);
    rig.press(Key::Down);
    assert!(rig.app.about_scroll > 0.0, "the license list scrolls");
    rig.press_all(&[Key::Up, Key::Up]);
    assert_eq!(rig.app.about_scroll, 0.0);
    rig.press(Key::Back);
    assert_eq!(rig.app.screen(), &Screen::Settings);
    assert_eq!(rig.focused(), Some(ids::SETTINGS_BASE + 3));
}

#[test]
fn the_editor_opens_from_the_settings_and_has_no_done_button() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    assert_eq!(
        rig.focused(),
        Some(WALLPAPER_BASE),
        "the active wallpaper card has the focus"
    );
    // There is no DONE button in the layout (4000).
    assert!(!rig.app.focus.focus(4000));
}

#[test]
fn back_in_the_editor_returns_to_the_settings_and_then_to_the_lume_tile() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    let effects = rig.press(Key::Back);
    assert_eq!(effects, [Effect::SaveConfig], "the changes are kept");
    assert_eq!(rig.app.screen(), &Screen::Settings);
    assert_eq!(rig.focused(), Some(ids::SETTINGS_BASE));
    // The way out retraces the way in: the "Lume" tile of the Apps tab.
    rig.press(Key::Back);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(rig.focused(), Some(ids::GRID_SETTINGS));
}

#[test]
fn back_in_the_editor_opened_from_the_home_screen_goes_home() {
    let mut rig = Rig::new();
    rig.app.config.favorites.clear();
    rig.app.go_home();
    rig.frame();
    rig.press(Key::Ok);
    assert_eq!(rig.app.screen(), &Screen::Editor);
    assert_eq!(rig.press(Key::Back), [Effect::SaveConfig]);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
}

#[test]
fn a_favorite_moves_left_and_the_focus_follows_it() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    // Spotify is in position 4 (index 3): its "move left" button.
    let left = FAVORITE_BUTTON_BASE + 3 * 3;
    assert_eq!(press_on(&mut rig, left), [Effect::SaveConfig]);
    assert_eq!(
        rig.app.config.favorites,
        [NETFLIX, YOUTUBE, SPOTIFY, PRIME, DISNEY]
    );
    assert_eq!(
        rig.focused(),
        Some(FAVORITE_BUTTON_BASE + 2 * 3),
        "on the same button of the moved card"
    );
}

#[test]
fn a_favorite_moves_right_and_the_edges_are_disabled() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    let enabled = |rig: &Rig, id: FocusId| {
        rig.app
            .focus
            .rows()
            .iter()
            .any(|r| r.items.iter().any(|i| i.id == id && i.enabled))
    };
    assert!(
        !enabled(&rig, FAVORITE_BUTTON_BASE),
        "the first cannot move left"
    );
    assert!(
        !enabled(&rig, FAVORITE_BUTTON_BASE + 4 * 3 + 1),
        "the last cannot move right"
    );
    let right = FAVORITE_BUTTON_BASE + 1;
    assert_eq!(press_on(&mut rig, right), [Effect::SaveConfig]);
    assert_eq!(rig.app.config.favorites[..2], [YOUTUBE, NETFLIX]);
}

#[test]
fn a_favorite_can_be_removed() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    let remove = FAVORITE_BUTTON_BASE + 2;
    assert_eq!(press_on(&mut rig, remove), [Effect::SaveConfig]);
    assert_eq!(rig.app.config.favorites, [YOUTUBE, PRIME, SPOTIFY, DISNEY]);
    // The removed app is offered in "Add app" again.
    assert!(rig.app.addable_apps().iter().any(|(p, _)| p == NETFLIX));
}

#[test]
fn adding_a_favorite_appends_it_and_takes_it_off_the_add_list() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    let addable = rig.app.addable_apps();
    let chrome = addable
        .iter()
        .position(|(p, _)| p == "com.demo.chrome")
        .unwrap() as u32;
    assert_eq!(
        press_on(&mut rig, ADD_APP_BASE + chrome),
        [Effect::SaveConfig]
    );
    assert_eq!(rig.app.config.favorites.last().unwrap(), "com.demo.chrome");
    assert!(!rig
        .app
        .addable_apps()
        .iter()
        .any(|(p, _)| p == "com.demo.chrome"));
}

#[test]
fn the_twelve_favorites_limit_disables_adding() {
    let catalog = many_apps(20);
    let mut rig = Rig::with_catalog(catalog.clone());
    rig.app.config.favorites = catalog
        .apps()
        .iter()
        .take(12)
        .map(|a| a.package.clone())
        .collect();
    open_editor(&mut rig);
    let focusable_adds = rig
        .app
        .focus
        .rows()
        .iter()
        .flat_map(|r| r.items.iter())
        .filter(|i| (ADD_APP_BASE..CONFIRM_RESTORE).contains(&i.id))
        .count();
    assert_eq!(focusable_adds, 0, "no \"Add app\" chip can take the focus");
    assert!(rig.app.config.favorites_full());
    assert!(!rig.app.config.add_favorite("com.demo.app19"));
}

#[test]
fn restoring_the_defaults_asks_first() {
    let mut rig = Rig::new();
    rig.app.config.remove_favorite(0);
    open_editor(&mut rig);
    assert_eq!(press_on(&mut rig, RESTORE), []);
    assert!(rig.app.editor.confirm, "the dialog is open");
    assert_eq!(
        rig.focused(),
        Some(CONFIRM_CANCEL),
        "the safe choice has the focus"
    );
    assert_eq!(rig.app.config.favorites.len(), 4, "nothing changed yet");
    // Only the dialog's buttons can take the focus.
    assert!(rig.app.focus.rows().iter().all(|r| r
        .items
        .iter()
        .all(|i| i.id == CONFIRM_RESTORE || i.id == CONFIRM_CANCEL)));
    rig.press(Key::Left);
    assert_eq!(rig.focused(), Some(CONFIRM_RESTORE));
    assert_eq!(rig.press(Key::Ok), [Effect::SaveConfig]);
    assert!(!rig.app.editor.confirm);
    assert_eq!(rig.app.config, HomeConfig::first_use(&rig.app.catalog));
}

#[test]
fn cancelling_the_restore_changes_nothing() {
    let mut rig = Rig::new();
    rig.app.config.remove_favorite(0);
    let before = rig.app.config.clone();
    open_editor(&mut rig);
    press_on(&mut rig, RESTORE);
    assert_eq!(rig.press(Key::Ok), [], "the focus starts on Cancel");
    assert_eq!(rig.app.config, before);
    assert!(!rig.app.editor.confirm);
    assert_eq!(rig.focused(), Some(RESTORE));
    // Back also closes the dialog without leaving the editor.
    press_on(&mut rig, RESTORE);
    assert_eq!(rig.press(Key::Back), []);
    assert_eq!(rig.app.screen(), &Screen::Editor);
    assert!(!rig.app.editor.confirm);
}

#[test]
fn restoring_keeps_the_usage_history() {
    let mut rig = Rig::new();
    rig.app.usage.record_open(NETFLIX, NOW);
    open_editor(&mut rig);
    press_on(&mut rig, RESTORE);
    rig.press_all(&[Key::Left, Key::Ok]);
    assert_eq!(rig.app.usage.opens_in_last_7_days(NETFLIX, NOW), 1);
}

#[test]
fn the_editor_scrolls_to_keep_the_focus_visible() {
    let catalog = many_apps(30);
    let mut rig = Rig::with_catalog(catalog.clone());
    rig.app.config.favorites = catalog
        .apps()
        .iter()
        .take(12)
        .map(|a| a.package.clone())
        .collect();
    open_editor(&mut rig);
    assert_eq!(rig.app.scroll_y, 0.0);
    focus_on(&mut rig, RESTORE);
    rig.advance(1000);
    assert!(
        rig.app.scroll_y > 100.0,
        "scrolled to the bottom button: {}",
        rig.app.scroll_y
    );
    let rect = rig.app.focus.current().unwrap().rect;
    assert!(
        rect.bottom() - rig.app.scroll_y <= 720.0,
        "the focused button is on screen"
    );
    // Back to the first favorite: the page returns to the top.
    focus_on(&mut rig, FAVORITE_BUTTON_BASE + 1);
    rig.advance(1000);
    assert_eq!(rig.app.scroll_y, 0.0);
}

#[test]
fn texts_in_the_editor_follow_the_language() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    let list = rig.frame();
    // The screen has plenty of text; a smoke check that it draws without panicking in English too.
    assert!(list.len() > 50);
    rig.app.translator = Translator::new(&["en"]);
    assert!(rig.frame().len() > 50);
    assert_eq!(
        rig.app
            .translator
            .get(crate::i18n::StringId::EditorWallpaper),
        "Wallpaper"
    );
}

#[test]
fn round_favorites_toggles_with_ok_and_emits_save_config() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    rig.press(Key::Down);
    assert_eq!(rig.focused(), Some(ROUND_FAVORITES));
    assert!(!rig.app.config.round_favorites, "square is the default");

    let effects = rig.press(Key::Ok);
    assert_eq!(effects, [Effect::SaveConfig]);
    assert!(rig.app.config.round_favorites);

    let effects = rig.press(Key::Ok);
    assert_eq!(effects, [Effect::SaveConfig]);
    assert!(!rig.app.config.round_favorites);
}

#[test]
fn restore_defaults_resets_round_favorites_to_false() {
    let mut rig = Rig::new();
    rig.app.config.round_favorites = true;
    open_editor(&mut rig);
    press_on(&mut rig, RESTORE);
    press_on(&mut rig, CONFIRM_RESTORE);
    assert!(!rig.app.config.round_favorites);
}

#[test]
fn editor_fits_six_cards_per_line_with_no_horizontal_overflow() {
    let mut rig = Rig::new();
    let catalog = many_apps(12);
    rig.app.catalog = catalog.clone();
    rig.app.config.favorites = catalog
        .apps()
        .iter()
        .take(12)
        .map(|a| a.package.clone())
        .collect();
    open_editor(&mut rig);
    let rows = rig.app.focus.rows();
    // Row 0 is the wallpaper cards row (one item per wallpaper)
    assert_eq!(
        rows[0].items.len(),
        crate::wallpaper::WallpaperId::ALL.len()
    );
    // Row 1 is ROUND_FAVORITES (1 item)
    assert_eq!(rows[1].items.len(), 1);
    assert_eq!(rows[1].items[0].id, ROUND_FAVORITES);
    // Row 2 is the first line of 6 cards * 3 buttons = 18 buttons
    assert_eq!(rows[2].items.len(), 18);
    // Row 3 is the second line of 6 cards * 3 buttons = 18 buttons
    assert_eq!(rows[3].items.len(), 18);
}

fn images(list: &DisplayList) -> usize {
    list.cmds()
        .iter()
        .filter(|c| matches!(c, crate::render::Cmd::Image(_)))
        .count()
}

fn glyphs(list: &DisplayList) -> usize {
    list.cmds()
        .iter()
        .filter(|c| matches!(c, crate::render::Cmd::Glyph(_)))
        .count()
}

#[test]
fn a_grid_tile_shows_the_icon_instead_of_its_name() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    let before = rig.frame();
    assert_eq!(images(&before), 0, "no icon is loaded yet");

    rig.app
        .icons
        .reserve(NETFLIX, crate::app_icons::Art::Icon)
        .unwrap();
    let after = rig.frame();
    assert_eq!(images(&after), 1);
    // The name inside the tile is gone; the one printed under it stays.
    assert!(glyphs(&after) < glyphs(&before));
}

#[test]
fn a_banner_in_the_grid_leaves_only_the_name_under_the_tile() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    let before = rig.frame();
    rig.app
        .icons
        .reserve(NETFLIX, crate::app_icons::Art::Banner)
        .unwrap();
    let after = rig.frame();
    assert_eq!(images(&after), 1);
    assert!(glyphs(&after) < glyphs(&before));
}

fn many_apps(count: usize) -> Catalog {
    let entries: Vec<Discovered> = (0..count)
        .map(|i| discovered(&format!("com.demo.app{i:02}"), &format!("App {i:02}")))
        .collect();
    Catalog::from_discovered(&entries, "app.lume.tv")
}

#[test]
fn an_idle_apps_grid_taller_than_the_screen_asks_for_no_frames() {
    let mut rig = Rig::with_catalog(many_apps(30));
    open_apps(&mut rig);
    rig.press(Key::Down);
    assert_eq!(rig.focused(), Some(ids::GRID_BASE));
    for step in 0..120 {
        rig.advance(100);
        let deadline = rig.app.next_deadline(rig.now);
        // The focus animation of the first second is normal.
        assert!(
            step < 10 || deadline.is_none_or(|at| at > rig.now + 1_000),
            "step {step}: still animating, next deadline {deadline:?} at {}",
            rig.now
        );
    }
}

#[test]
fn the_focus_outline_of_a_tile_with_a_banner_runs_under_the_banner_edge() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    let focused = rig.app.catalog.apps()[0].package.clone();
    rig.app
        .icons
        .reserve(&focused, crate::app_icons::Art::Banner)
        .unwrap();
    rig.press(Key::Down);
    rig.advance(1000);
    let frame = rig.frame();
    let ring = frame
        .cmds()
        .iter()
        .find_map(|cmd| match cmd {
            Cmd::Rect(r) if matches!(r.kind, crate::render::RectKind::Ring { .. }) => Some(*r),
            _ => None,
        })
        .expect("the focus outline");
    let crate::render::RectKind::Ring { width, .. } = ring.kind else {
        unreachable!()
    };
    let banner = frame
        .cmds()
        .iter()
        .find_map(|cmd| match cmd {
            Cmd::Image(t) if t.background.a > 0 => Some(*t),
            _ => None,
        })
        .expect("the banner");
    // The outline is drawn first and reaches under the banner past where the banner turns opaque
    // (1.5 ramps of the buffer in, and the outline's own half ramp; the rig is at 720p, one canvas
    // unit per pixel), so the clipped edge blends with the blue and not with the wallpaper.
    let inner_left = ring.rect.x + width;
    let under = inner_left - banner.dest.x;
    assert!(under >= 2.0 * vars::EDGE_SOFTNESS - 1e-3, "{under}");
    // Seen from outside, the outline still starts where it did: its 2 units out of the grown tile.
    assert!((banner.dest.x - ring.rect.x - 2.0).abs() < 1e-3);
}

#[test]
fn a_tile_with_a_banner_is_one_shape_clipped_by_the_tile_corners() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.app
        .icons
        .reserve(NETFLIX, crate::app_icons::Art::Banner)
        .unwrap();
    let color = rig.app.catalog.get(NETFLIX).unwrap().color;
    let frame = rig.frame();
    let banner = frame
        .cmds()
        .iter()
        .find_map(|cmd| match cmd {
            Cmd::Image(t) if t.background == color => Some(*t),
            _ => None,
        })
        .expect("the banner over the tile color");
    // The corners are the tile's, cut by the shader at the final size (not baked in the banner).
    assert!(banner.radius >= THEME.tile.base.border_radius.unwrap());
    let fills_under = frame
        .cmds()
        .iter()
        .filter(|cmd| {
            matches!(cmd, Cmd::Rect(r) if matches!(r.kind, crate::render::RectKind::Fill(_))
                && r.rect == banner.dest)
        })
        .count();
    assert_eq!(fills_under, 0, "no second edge under the banner");
}

#[test]
fn the_apps_grid_is_drawn_in_few_batches_however_many_tiles_it_has() {
    // Tiles are drawn in passes (boxes, images, text): alternating the programs tile by tile
    // made one draw call per kind for every tile, which a TV GPU takes a toll on.
    let mut few = Rig::with_catalog(many_apps(5));
    let mut many = Rig::with_catalog(many_apps(15));
    for rig in [&mut few, &mut many] {
        open_apps(rig);
        for i in 0..rig.app.catalog.apps().len() {
            let package = rig.app.catalog.apps()[i].package.clone();
            let art = if i % 2 == 0 {
                crate::app_icons::Art::Banner
            } else {
                crate::app_icons::Art::Icon
            };
            rig.app.icons.reserve(&package, art).unwrap();
        }
    }
    let batches = |rig: &mut Rig| crate::render::Frame::build(&rig.frame()).batches.len();
    let (small, large) = (batches(&mut few), batches(&mut many));
    assert_eq!(small, large, "ten more tiles must not add draw calls");
    // The tile color drawn with the image (one edge per tile) did not add any: 14 before and after.
    assert!(large <= 14, "{large} draw calls");
}

// ---- top bar focus ------------------------------------------------------------------------------

/// The blue underlines of the tabs: left edge and opacity of each.
fn underlines(list: &DisplayList) -> Vec<(f32, f32)> {
    list.cmds()
        .iter()
        .filter_map(|cmd| match cmd {
            crate::render::Cmd::Rect(r)
                if r.rect.y == 64.0
                    && r.rect.w > 10.0
                    && r.rect.h == crate::theme::vars::TAB_UNDERLINE
                    && matches!(r.kind, crate::render::RectKind::Fill(_)) =>
            {
                Some((r.rect.x, r.opacity))
            }
            _ => None,
        })
        .collect()
}

/// Focus rings drawn inside the top bar's tab row (tabs and gear), excluding the glass rim.
fn top_bar_rings(list: &DisplayList) -> Vec<crate::render::RectCmd> {
    list.cmds()
        .iter()
        .filter_map(|cmd| match cmd {
            crate::render::Cmd::Rect(r)
                if r.rect.bottom() <= 88.0
                    && matches!(r.kind, crate::render::RectKind::Ring { width, .. } if width >= 2.0) =>
            {
                Some(*r)
            }
            _ => None,
        })
        .collect()
}

fn underline_alpha(list: &DisplayList) -> Vec<(f32, u8)> {
    list.cmds()
        .iter()
        .filter_map(|cmd| match cmd {
            crate::render::Cmd::Rect(r)
                if r.rect.y == 64.0
                    && r.rect.w > 10.0
                    && r.rect.h == crate::theme::vars::TAB_UNDERLINE =>
            {
                match r.kind {
                    crate::render::RectKind::Fill(crate::theme::Paint::Solid(c)) => {
                        Some((r.rect.x, c.a))
                    }
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_focused_tab_has_no_ring() {
    let mut rig = Rig::new();
    rig.press(Key::Up);
    assert_eq!(rig.focused(), Some(ids::TAB_HOME));
    rig.advance(1000);
    assert!(top_bar_rings(&rig.frame()).is_empty());
    rig.press(Key::Right);
    rig.advance(1000);
    assert_eq!(rig.focused(), Some(ids::TAB_APPS));
    assert!(top_bar_rings(&rig.frame()).is_empty());
}

#[test]
fn the_underline_follows_the_focus_on_the_tabs_and_the_screen_changes_with_it() {
    let mut rig = Rig::new();
    rig.press(Key::Up);
    rig.advance(1000);
    let lines = underline_alpha(&rig.frame());
    assert_eq!(lines.len(), 1, "on the active tab: {lines:?}");
    let home_x = rig.app_tab_text_x(Tab::Home) - 2.0;
    assert!((lines[0].0 - home_x).abs() < 0.5);

    // Focus on the other tab: the underline moves there and the screen changes at once, no OK.
    rig.press(Key::Right);
    rig.advance(1000);
    assert_eq!(rig.focused(), Some(ids::TAB_APPS));
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    let lines = underline_alpha(&rig.frame());
    assert_eq!(lines.len(), 1, "{lines:?}");
    let apps_x = rig.app_tab_text_x(Tab::Apps) - 2.0;
    assert!((lines[0].0 - apps_x).abs() < 0.5, "{lines:?}");

    // The gear: the underline stays under the tab that is showing.
    rig.press_all(&[Key::Right, Key::Right]);
    rig.advance(1000);
    assert_eq!(rig.focused(), Some(ids::GEAR));
    let lines = underline_alpha(&rig.frame());
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!((lines[0].0 - apps_x).abs() < 0.5);

    // Back to the first tab: it shows Home again, and the underline follows.
    rig.press_all(&[Key::Left, Key::Left, Key::Left]);
    rig.advance(1000);
    assert_eq!(rig.focused(), Some(ids::TAB_HOME));
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    let lines = underline_alpha(&rig.frame());
    assert_eq!(lines.len(), 1);
    assert!((lines[0].0 - home_x).abs() < 0.5);

    // Down from a tab enters the content of the tab that is showing.
    rig.press(Key::Down);
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE));
}

#[test]
fn the_underline_moves_with_a_transition() {
    let mut rig = Rig::new();
    rig.press(Key::Up);
    rig.advance(1000);
    rig.app.handle_key(Key::Right, rig.now);
    rig.advance(10);
    rig.advance(60);
    let lines = underline_alpha(&rig.frame());
    assert_eq!(lines.len(), 2, "both fade during the move: {lines:?}");
    assert!(lines.iter().all(|(_, a)| *a > 0 && *a < 255), "{lines:?}");
}

#[test]
fn the_focused_gear_has_a_blue_ring() {
    let mut rig = Rig::new();
    rig.press_all(&[Key::Up, Key::Right, Key::Right, Key::Right]);
    assert_eq!(rig.focused(), Some(ids::GEAR));
    rig.advance(1000);
    let rings = top_bar_rings(&rig.frame());
    assert_eq!(rings.len(), 1, "{rings:?}");
    let crate::render::RectKind::Ring { color, .. } = rings[0].kind else {
        unreachable!()
    };
    assert_eq!(
        (color.r, color.g, color.b),
        (
            crate::theme::vars::ACCENT.r,
            crate::theme::vars::ACCENT.g,
            crate::theme::vars::ACCENT.b
        )
    );
}

impl Rig {
    /// Where the label of `tab` starts in the top bar.
    fn app_tab_text_x(&mut self, tab: Tab) -> f32 {
        let tabs = topbar::BarLayout::new(
            &mut Painter {
                list: &mut DisplayList::new(),
                text: &mut self.text,
                display: self.display,
            },
            &self.app.translator.get(crate::i18n::StringId::TabHome),
            &self.app.translator.get(crate::i18n::StringId::TabApps),
            &self.app.clock,
        );
        tabs.text[tab as usize].0
    }
}

#[test]
fn down_from_the_favorites_goes_to_done_and_restore_is_to_its_left() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    assert_eq!(rig.focused(), Some(WALLPAPER_BASE));
    // Down goes to the round favorites switch.
    rig.press(Key::Down);
    assert_eq!(rig.focused(), Some(ROUND_FAVORITES));
    // Down goes to the favorite cards.
    rig.press(Key::Down);
    assert!(
        rig.focused()
            .is_some_and(|id| (FAVORITE_BUTTON_BASE..ADD_APP_BASE).contains(&id)),
        "the cards come after round favorites"
    );
    // Down from the cards goes to the "Add app" chips.
    rig.press(Key::Down);
    assert!(
        rig.focused().is_some_and(|id| id >= ADD_APP_BASE),
        "the chips come after cards"
    );
    rig.press_all(&[Key::Down, Key::Down]);
    assert_eq!(rig.focused(), Some(RESTORE));
    let restore_x = rig.app.focus.current().unwrap().rect.x;
    assert!(restore_x < 100.0, "at the left margin");
    // Up from "Restore" returns to the "Add app" chips.
    rig.press(Key::Up);
    assert!(rig.focused().is_some_and(|id| id >= ADD_APP_BASE));
    // The first card cannot move left: that button takes no focus.
    assert!(!rig.app.focus.focus(FAVORITE_BUTTON_BASE));
}

#[test]
fn from_an_empty_home_the_add_favorites_item_opens_the_editor_at_the_favorites() {
    let mut rig = Rig::new();
    rig.app.config.favorites.clear();
    rig.app.go_home();
    rig.frame();
    assert_eq!(rig.focused(), Some(ids::FAVORITES_ADD));
    rig.press(Key::Ok);
    assert_eq!(rig.app.screen(), &Screen::Editor);
    // The focus starts on active wallpaper card.
    assert_eq!(rig.focused(), Some(WALLPAPER_BASE));
    // Down goes to the round favorites switch.
    rig.press(Key::Down);
    assert_eq!(rig.focused(), Some(ROUND_FAVORITES));
    // Down through chips: lands on "Restore defaults".
    rig.press_all(&[Key::Down, Key::Down, Key::Down, Key::Down]);
    assert_eq!(rig.focused(), Some(RESTORE));
}

#[test]
fn selecting_wallpaper_in_editor_changes_wallpaper_and_saves() {
    let mut rig = Rig::new();
    open_editor(&mut rig);
    assert_eq!(
        rig.app.config.wallpaper,
        crate::wallpaper::WallpaperId::Cinematic
    );
    // Focus the fifth wallpaper card (Starry Mountains).
    focus_on(&mut rig, WALLPAPER_BASE + 4);
    let effects = rig.press(Key::Ok);
    assert_eq!(effects, [Effect::SaveConfig]);
    assert_eq!(
        rig.app.config.wallpaper,
        crate::wallpaper::WallpaperId::MountainStars
    );
}

// ---- wallpaper --------------------------------------------------------------------------------

/// A fill of the whole visible area: what the screens used to paint as their background.
fn full_screen_fills(list: &DisplayList, visible: Rect) -> usize {
    list.cmds()
        .iter()
        .filter(|cmd| {
            matches!(cmd, crate::render::Cmd::Rect(r)
                if matches!(r.kind, crate::render::RectKind::Fill(_))
                    && r.rect.w >= visible.w
                    && r.rect.h >= visible.h)
        })
        .count()
}

#[test]
fn on_other_proportions_the_wallpaper_covers_the_whole_surface_without_stretching() {
    for (w, h) in [(1920, 1200), (2560, 1080), (1024, 768)] {
        let display = Display::new(w, h);
        let mut list = DisplayList::new();
        let mut text = TextEngine::new(display.scale);
        let mut painter = Painter {
            list: &mut list,
            text: &mut text,
            display,
        };
        painter.wallpaper(false);
        let Some(crate::render::Cmd::Wallpaper(quad)) = list.cmds().first() else {
            panic!()
        };
        assert_eq!(quad.dest, display.visible_rect(), "{w}x{h}");
        // The shown part keeps the proportion of the surface: nothing is stretched.
        let image = crate::wallpaper::current();
        let shown = (quad.uv.w * image.width as f32) / (quad.uv.h * image.height as f32);
        assert!(
            (shown - w as f32 / h as f32).abs() < 1e-3,
            "{w}x{h}: {shown}"
        );
        assert!(
            quad.uv.x >= 0.0
                && quad.uv.right() <= 1.0001
                && quad.uv.y >= 0.0
                && quad.uv.bottom() <= 1.0001
        );
    }
}

// ---- contrast over the wallpaper ----------------------------------------------------------------

fn relative_luminance(rgb: [f32; 3]) -> f32 {
    let linear = |c: f32| {
        let c = c / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(rgb[0]) + 0.7152 * linear(rgb[1]) + 0.0722 * linear(rgb[2])
}

fn contrast(text: crate::theme::Color, background_luminance: f32) -> f32 {
    let text = relative_luminance([text.r, text.g, text.b].map(f32::from));
    (text.max(background_luminance) + 0.05) / (text.min(background_luminance) + 0.05)
}

/// The brightest texel of the wallpaper inside `area` (logical units of the 1280x720 canvas),
/// after the layer `dim` (if any) is drawn over it.
fn brightest_behind(
    wallpaper: &crate::wallpaper::Wallpaper,
    area: Rect,
    dim: Option<crate::theme::Color>,
) -> f32 {
    let (sx, sy) = (
        wallpaper.width as f32 / 1280.0,
        wallpaper.height as f32 / 720.0,
    );
    let rgb = wallpaper.decompress_rgb();
    let mut brightest = 0.0f32;
    for y in (area.y * sy) as usize
        ..((area.bottom() * sy).ceil() as usize).min(wallpaper.height as usize)
    {
        for x in (area.x * sx) as usize
            ..((area.right() * sx).ceil() as usize).min(wallpaper.width as usize)
        {
            let i = (y * wallpaper.width as usize + x) * 3;
            let mut pixel = [0, 1, 2].map(|c| f32::from(rgb[i + c]));
            if let Some(dim) = dim {
                let a = f32::from(dim.a) / 255.0;
                let shade = [dim.r, dim.g, dim.b].map(f32::from);
                for c in 0..3 {
                    pixel[c] = pixel[c] * (1.0 - a) + shade[c] * a;
                }
            }
            brightest = brightest.max(relative_luminance(pixel));
        }
    }
    brightest
}

#[test]
fn text_on_the_dimmed_screens_has_4_5_to_1_over_the_brightest_part_of_the_wallpaper() {
    let everywhere = Rect::new(0.0, 0.0, 1280.0, 720.0);
    for id in crate::wallpaper::WallpaperId::ALL {
        let w = id.wallpaper();
        let behind = brightest_behind(w, everywhere, Some(painter::TEXT_SCREEN_DIM));
        // The dimmest text color used on those screens (tile names, breadcrumbs, hints).
        let ratio = contrast(crate::theme::vars::TEXT_MUTED, behind);
        assert!(
            ratio >= 4.5,
            "{}: muted text: {ratio:.2}:1 over luminance {behind:.4}",
            id.as_str()
        );
        assert!(
            contrast(crate::theme::vars::TEXT, behind) >= 4.5,
            "{}: normal text contrast over luminance {behind:.4}",
            id.as_str()
        );
    }
}

#[test]
#[ignore = "the wallpapers no longer carry the top darkening (author's decision, 2026-10-21; see CLAUDE.md): the top bar text does not reach 4.5:1 over the bright ones; see the wallpapers-full-hd change"]
fn the_top_bar_texts_have_4_5_to_1_over_the_undimmed_wallpaper_on_home() {
    let mut rig = Rig::new();
    let tabs = topbar::BarLayout::new(
        &mut Painter {
            list: &mut DisplayList::new(),
            text: &mut rig.text,
            display: rig.display,
        },
        &rig.app.translator.get(crate::i18n::StringId::TabHome),
        &rig.app.translator.get(crate::i18n::StringId::TabApps),
        &rig.app.clock,
    );
    let (home, apps) = (tabs.rect(Tab::Home), tabs.rect(Tab::Apps));
    let tab_area = Rect::new(home.x, home.y, apps.right() - home.x, home.h);
    for id in crate::wallpaper::WallpaperId::ALL {
        let w = id.wallpaper();
        let muted = brightest_behind(w, tab_area, None);
        let ratio = contrast(crate::theme::vars::TEXT_MUTED, muted);
        assert!(
            ratio >= 4.5,
            "{}: inactive tab: {ratio:.2}:1 over luminance {muted:.4}",
            id.as_str()
        );
        // The white texts: the wordmark, the active tab, the gear and the clock.
        for (name, area) in [
            ("logo", Rect::new(64.0, 20.0, 120.0, 44.0)),
            ("tabs", tab_area),
            ("gear", Rect::new(1096.0, 20.0, 48.0, 48.0)),
            ("clock", Rect::new(1150.0, 28.0, 70.0, 30.0)),
        ] {
            let ratio = contrast(crate::theme::vars::TEXT, brightest_behind(w, area, None));
            assert!(ratio >= 4.5, "{}: {name}: {ratio:.2}:1", id.as_str());
        }
    }
}

// ---- the dock -----------------------------------------------------------------------------------

fn rings_in(list: &DisplayList, area: Rect) -> usize {
    list.cmds()
        .iter()
        .filter(|cmd| {
            matches!(cmd, crate::render::Cmd::Rect(r)
                if matches!(r.kind, crate::render::RectKind::Ring { .. }) && area.intersects(&r.rect))
        })
        .count()
}

fn favorites_dock(rig: &mut Rig, count: usize) {
    let packages: Vec<String> = rig
        .app
        .catalog
        .apps()
        .iter()
        .take(count)
        .map(|a| a.package.clone())
        .collect();
    rig.app.config.favorites = packages;
    rig.press(Key::Home);
    rig.advance(1000);
}

#[test]
fn a_focused_dock_item_has_a_glass_edge_inside_the_bar() {
    let mut rig = Rig::new();
    rig.press_all(&[Key::Right, Key::Right]);
    rig.advance(1000);
    let list = rig.frame();
    let dock_area = Rect::new(0.0, 400.0, 1280.0, 320.0);
    assert_eq!(
        rings_in(&list, dock_area),
        2,
        "the bar and focused icon each have a thin reflective edge"
    );
    // The reflective edge stays within the bar.
    assert!(top_bar_rings(&list).is_empty());
}

#[test]
fn ok_on_a_dock_icon_opens_its_app() {
    let mut rig = Rig::new();
    rig.press_all(&[Key::Right, Key::Right, Key::Right]);
    let effects = rig.press(Key::Ok);
    assert_eq!(
        effects[0],
        Effect::OpenApp {
            package: SPOTIFY.into(),
            activity: format!("{SPOTIFY}.Main")
        }
    );
}

#[test]
fn an_idle_home_asks_for_no_frame_but_the_clocks() {
    let mut rig = Rig::new();
    rig.press_all(&[Key::Right, Key::Right]);
    rig.advance(1000);
    let now = rig.now;
    let deadline = rig.app.next_deadline(now).expect("the clock");
    assert_eq!(
        deadline,
        now + rig.app.clock.millis_to_next_minute(),
        "only the clock turning"
    );
    // And after a long wait the frame still has the same few things to draw.
    let busy = rig.frame().len();
    rig.advance(30_000);
    assert_eq!(rig.frame().len(), busy);
    assert_eq!(
        rig.app.next_deadline(rig.now),
        Some(rig.now + rig.app.clock.millis_to_next_minute())
    );
}

// ---- coming back from an opened app ----------------------------------------------------------------

/// What happens to lume when an app takes the screen and the user comes back with Home.
fn leave_and_come_back(rig: &mut Rig) {
    rig.app.went_to_background();
    rig.app.came_to_foreground();
    rig.press(Key::Home);
}

fn grid_id_of(rig: &Rig, package: &str) -> FocusId {
    let index = rig
        .app
        .catalog
        .apps()
        .iter()
        .position(|a| a.package == package)
        .unwrap();
    ids::GRID_BASE + index as u32
}

#[test]
fn back_from_an_app_opened_in_the_apps_tab_returns_to_that_tab_and_icon() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    let youtube = grid_id_of(&rig, YOUTUBE);
    focus_on(&mut rig, youtube);
    let effects = rig.press(Key::Ok);
    assert!(matches!(&effects[0], Effect::OpenApp { package, .. } if package == YOUTUBE));
    // The user comes back with the Home button.
    leave_and_come_back(&mut rig);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(rig.focused(), Some(youtube));
}

#[test]
fn back_from_an_app_opened_from_the_dock_returns_to_that_icon() {
    let mut rig = Rig::new();
    rig.press_all(&[Key::Right, Key::Right, Key::Right]);
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE + 3));
    rig.press(Key::Ok);
    leave_and_come_back(&mut rig);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(
        rig.focused(),
        Some(ids::FAVORITE_BASE + 3),
        "not the first icon"
    );
}

#[test]
fn the_grid_scroll_is_kept_when_coming_back() {
    let mut rig = Rig::with_catalog(many_apps(40));
    open_apps(&mut rig);
    rig.press_all(&[Key::Down; 6]);
    rig.advance(1000);
    let (focus, scroll) = (rig.focused().unwrap(), rig.app.scroll_y);
    assert!(scroll > 0.0, "the grid scrolled");
    rig.press(Key::Ok);
    leave_and_come_back(&mut rig);
    rig.advance(1000);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(rig.focused(), Some(focus));
    assert!(
        (rig.app.scroll_y - scroll).abs() < 1.0,
        "{} vs {scroll}",
        rig.app.scroll_y
    );
    let rect = rig.app.focus.current().unwrap().rect;
    assert!(rect.y - rig.app.scroll_y >= 76.0 && rect.bottom() - rig.app.scroll_y <= 720.0);
}

#[test]
fn back_from_the_settings_opened_by_the_lume_tile_returns_to_the_tile() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    focus_on(&mut rig, ids::GRID_SETTINGS);
    rig.press(Key::Ok);
    rig.press(Key::Back);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
}

#[test]
fn coming_back_from_the_android_settings_keeps_the_gear_focused() {
    let mut rig = Rig::new();
    rig.press_all(&[Key::Up, Key::Right, Key::Right, Key::Right]);
    assert_eq!(rig.press(Key::Ok), [Effect::OpenAndroidSettings]);
    leave_and_come_back(&mut rig);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(rig.focused(), Some(ids::GEAR));
}

#[test]
fn any_other_key_forgets_the_place_so_home_goes_to_the_start() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.press(Key::Ok);
    // lume came back by Back (no Home press): the screen is already as it was, and the user moves.
    rig.press(Key::Right);
    rig.press(Key::Home);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(
        rig.focused(),
        Some(ids::FAVORITE_BASE),
        "the usual Home press"
    );
}

#[test]
fn home_without_an_app_having_been_opened_goes_to_the_start() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press_all(&[Key::Down, Key::Right]);
    rig.press(Key::Home);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE));
}

#[test]
fn the_place_is_used_once() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.press(Key::Ok);
    leave_and_come_back(&mut rig);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    rig.advance(5000);
    rig.press(Key::Home);
    assert_eq!(
        rig.app.screen(),
        &Screen::Main(Tab::Home),
        "the second press is the usual one"
    );
}

#[test]
fn an_app_that_could_not_be_opened_leaves_nothing_to_go_back_to() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.press(Key::Ok);
    rig.app.app_not_found(rig.now);
    rig.press(Key::Home);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
}

#[test]
fn home_pressed_while_lume_is_on_screen_goes_to_the_start_even_after_opening_an_app() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.press(Key::Ok);
    // lume never left the screen (the app did not open, or the user came back some other way).
    rig.press(Key::Home);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE));
}

#[test]
fn a_home_press_long_after_coming_back_is_the_usual_one() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.press(Key::Ok);
    rig.app.went_to_background();
    rig.app.came_to_foreground();
    rig.advance(100); // the first frame back
    rig.advance(5000);
    rig.press(Key::Home);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE));
}

#[test]
fn a_home_press_that_arrives_just_after_the_first_frame_back_still_restores() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    let focus = rig.focused();
    rig.press(Key::Ok);
    rig.app.went_to_background();
    rig.app.came_to_foreground();
    rig.advance(100);
    rig.press(Key::Home);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(rig.focused(), focus);
}

/// The horizontal center of the name bubble above the dock icons, if it is drawn.
fn bubble_center(list: &DisplayList) -> Option<f32> {
    list.cmds().iter().find_map(|cmd| match cmd {
        crate::render::Cmd::Rect(r)
            if r.rect.h == 40.0
                && r.rect.y > 400.0
                && matches!(r.kind, crate::render::RectKind::Fill(_)) =>
        {
            Some(r.rect.center().0)
        }
        _ => None,
    })
}

#[test]
fn moving_the_focus_to_a_tab_shows_it_without_ok_and_ok_on_it_changes_nothing() {
    let mut rig = Rig::new();
    rig.press(Key::Up);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    rig.press(Key::Right);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps), "no OK needed");
    assert_eq!(
        rig.focused(),
        Some(ids::TAB_APPS),
        "the focus stays on the tab"
    );
    let labels_before = glyphs(&rig.frame());
    assert!(rig.press(Key::Ok).is_empty());
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(glyphs(&rig.frame()), labels_before);
    // And left shows Home again.
    rig.press(Key::Left);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(rig.focused(), Some(ids::TAB_HOME));
}

#[test]
fn the_apps_tab_scrolled_back_to_the_top_when_the_focus_arrives_on_it() {
    let mut rig = Rig::with_catalog(many_apps(40));
    open_apps(&mut rig);
    rig.press_all(&[Key::Down; 6]);
    rig.advance(1000);
    assert!(rig.app.scroll_y > 0.0);
    rig.press_all(&[Key::Up; 7]);
    assert_eq!(rig.focused(), Some(ids::TAB_APPS));
    rig.press(Key::Left);
    rig.press(Key::Right);
    rig.advance(1000);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(rig.app.scroll_y, 0.0);
}

/// The color of the (only) tab underline in the frame.
fn underline_color(list: &DisplayList) -> crate::theme::Color {
    list.cmds()
        .iter()
        .find_map(|cmd| match cmd {
            crate::render::Cmd::Rect(r)
                if r.rect.y == 64.0
                    && r.rect.w > 10.0
                    && r.rect.h == crate::theme::vars::TAB_UNDERLINE =>
            {
                match r.kind {
                    crate::render::RectKind::Fill(crate::theme::Paint::Solid(c)) => Some(c),
                    _ => None,
                }
            }
            _ => None,
        })
        .expect("an underline is drawn")
}

fn rgb(c: crate::theme::Color) -> [u8; 3] {
    [c.r, c.g, c.b]
}

#[test]
fn the_underline_fades_between_white_and_blue_with_the_focus() {
    let mut rig = Rig::new();
    rig.advance(1000);
    rig.app.handle_key(Key::Up, rig.now);
    rig.advance(10);
    rig.advance(60);
    let c = underline_color(&rig.frame());
    let (white, blue) = (crate::theme::vars::TEXT, crate::theme::vars::ACCENT);
    assert!(
        c.r < white.r && c.r > blue.r,
        "between white and blue: {c:?}"
    );
}

#[test]
fn in_the_apps_grid_the_underline_is_white_too() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.advance(1000);
    assert_eq!(rig.focused(), Some(ids::GRID_BASE));
    assert_eq!(
        rgb(underline_color(&rig.frame())),
        rgb(crate::theme::vars::TEXT)
    );
}

#[test]
fn changing_the_tab_fades_the_new_content_in_and_then_goes_idle() {
    let mut rig = Rig::new();
    rig.press(Key::Up);
    rig.advance(1000);
    rig.app.handle_key(Key::Right, rig.now);
    rig.advance(10);
    let mid = rig.app.anim.get(super::Anim::TabFade, 1.0, rig.now);
    assert!(mid > 0.0 && mid < 1.0, "mid-fade: {mid}");
    assert!(
        rig.app.is_animating(rig.now),
        "frames are asked for while it fades"
    );
    // The content is drawn translucent; the top bar after it is not.
    let list = rig.frame();
    assert!(
        list.cmds()
            .iter()
            .any(|c| matches!(c, crate::render::Cmd::Rect(r) if r.opacity < 1.0)),
        "some content is translucent during the fade"
    );
    rig.advance(1000);
    assert!(
        !rig.app.is_animating(rig.now),
        "nothing animates once it is done"
    );
}

#[test]
fn the_name_under_the_focused_tile_is_white_and_the_others_are_muted() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    rig.advance(1000);
    let focused = rig.app.focus_progress(rig.focused().unwrap());
    assert_eq!(focused, 1.0);
    let mut style = |progress: f32| {
        let p = Painter {
            list: &mut DisplayList::new(),
            text: &mut rig.text,
            display: rig.display,
        };
        p.focus_style(&crate::theme::THEME.tile_label, progress)
            .color
    };
    assert_eq!(style(0.0), crate::theme::vars::TEXT_MUTED);
    assert_eq!(style(1.0), crate::theme::vars::TEXT);
}

// ---- TV statistics panel ---------------------------------------------------------------------

fn sample_stats() -> crate::stats::Stats {
    crate::stats::Stats::parse("2000000\t800000\t12000000000\t7000000000\tstat,100,1000").unwrap()
}

#[test]
fn the_statistics_are_read_only_while_the_settings_are_on_screen() {
    let mut rig = Rig::new();
    assert_eq!(rig.app.stats_deadline(rig.now), None, "the home screen");
    open_apps(&mut rig);
    assert_eq!(rig.app.stats_deadline(rig.now), None, "the apps screen");
    rig.press(Key::Down);
    press_on(&mut rig, ids::GRID_SETTINGS);
    assert_eq!(rig.app.screen(), &Screen::Settings);
    assert_eq!(
        rig.app.stats_deadline(rig.now),
        Some(rig.now),
        "due at once"
    );

    rig.app.apply_stats(sample_stats(), rig.now);
    assert_eq!(rig.app.stats_deadline(rig.now), Some(rig.now + 2000));
    assert_eq!(
        rig.app.cpu_use, None,
        "the first reading has nothing to compare with"
    );

    // Leaving for the About screen, the editor or the background stops the readings.
    rig.press_all(&[Key::Down, Key::Down, Key::Down, Key::Ok]);
    assert_eq!(rig.app.screen(), &Screen::About);
    assert_eq!(rig.app.stats_deadline(rig.now), None);
    rig.press(Key::Back);
    assert!(rig.app.stats_deadline(rig.now).is_some());
    rig.app.went_to_background();
    assert_eq!(rig.app.stats_deadline(rig.now), None);
}

#[test]
fn the_cpu_comes_from_two_readings() {
    let mut rig = Rig::new();
    open_settings(&mut rig);
    rig.app.apply_stats(sample_stats(), 0);
    let later =
        crate::stats::Stats::parse("2000000\t800000\t12000000000\t7000000000\tstat,400,2000")
            .unwrap();
    rig.app.apply_stats(later, 2000);
    assert!((rig.app.cpu_use.unwrap() - 0.3).abs() < 1e-6);
}

// ---- the context menu (the Menu button) ------------------------------------------------------

use super::MenuAction;

fn menu_actions(rig: &Rig) -> Vec<MenuAction> {
    let menu = rig.app.context_menu().expect("a menu is open");
    menu.items.iter().map(|i| i.action).collect()
}

fn selected_action(rig: &Rig) -> MenuAction {
    let menu = rig.app.context_menu().expect("a menu is open");
    menu.items[menu.selected].action
}

/// Opens the Apps tab and the menu of the app `package`.
fn open_grid_menu(rig: &mut Rig, package: &str) -> FocusId {
    open_apps(rig);
    let id = grid_id_of(rig, package);
    focus_on(rig, id);
    rig.press(Key::Menu);
    id
}

/// Opens the menu of the favorite at `index` of the dock.
fn open_dock_menu(rig: &mut Rig, index: usize) {
    focus_on(rig, ids::FAVORITE_BASE + index as u32);
    rig.press(Key::Menu);
}

#[test]
fn menu_on_an_app_of_the_grid_opens_with_the_focus_on_the_first_item() {
    let mut rig = Rig::new();
    let chrome = open_grid_menu(&mut rig, "com.demo.chrome");
    assert_eq!(
        menu_actions(&rig),
        [
            MenuAction::AddToHome,
            MenuAction::AppInfo,
            MenuAction::Uninstall
        ]
    );
    assert_eq!(selected_action(&rig), MenuAction::AddToHome);
    assert_eq!(
        rig.focused(),
        Some(chrome),
        "the tile keeps the focus under the menu"
    );
}

#[test]
fn menu_does_nothing_on_the_top_bar_the_lume_tile_add_favorites_and_the_settings() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    focus_on(&mut rig, ids::GEAR);
    assert!(rig.press(Key::Menu).is_empty());
    assert!(rig.app.context_menu().is_none());
    assert_eq!(rig.focused(), Some(ids::GEAR));

    focus_on(&mut rig, ids::GRID_SETTINGS);
    rig.press(Key::Menu);
    assert!(rig.app.context_menu().is_none());

    rig.press(Key::Ok);
    assert_eq!(rig.app.screen(), &Screen::Settings);
    rig.press(Key::Menu);
    assert!(rig.app.context_menu().is_none());
    assert_eq!(rig.app.screen(), &Screen::Settings);

    let mut rig = Rig::new();
    rig.app.config.favorites.clear();
    rig.advance(0);
    focus_on(&mut rig, ids::FAVORITES_ADD);
    rig.press(Key::Menu);
    assert!(rig.app.context_menu().is_none());
}

#[test]
fn back_or_menu_close_the_menu_without_changing_anything() {
    for key in [Key::Back, Key::Menu] {
        let mut rig = Rig::new();
        let chrome = open_grid_menu(&mut rig, "com.demo.chrome");
        let favorites = rig.app.config.favorites.clone();
        assert!(rig.press(key).is_empty());
        assert!(rig.app.context_menu().is_none(), "{key:?}");
        assert_eq!(rig.app.config.favorites, favorites);
        assert_eq!(rig.focused(), Some(chrome));
        assert_eq!(
            rig.app.screen(),
            &Screen::Main(Tab::Apps),
            "Back did not leave the tab"
        );
    }
}

#[test]
fn the_focus_is_trapped_in_the_menu() {
    let mut rig = Rig::new();
    let chrome = open_grid_menu(&mut rig, "com.demo.chrome");
    rig.press_all(&[Key::Down, Key::Down, Key::Down, Key::Down]);
    assert_eq!(
        selected_action(&rig),
        MenuAction::Uninstall,
        "stops at the last item"
    );
    rig.press_all(&[Key::Left, Key::Right]);
    assert!(rig.app.context_menu().is_some());
    assert_eq!(rig.focused(), Some(chrome), "the grid did not move");
    rig.press_all(&[Key::Up, Key::Up, Key::Up, Key::Up]);
    assert_eq!(selected_action(&rig), MenuAction::AddToHome);
    assert_eq!(
        rig.focused(),
        Some(chrome),
        "nor did the top bar take the focus"
    );
}

#[test]
fn the_menu_fades_in_and_out_in_150_ms() {
    let mut rig = Rig::new();
    open_grid_menu(&mut rig, "com.demo.chrome");
    rig.advance(200);
    assert_eq!(rig.app.anim.get(Anim::MenuFade, 1.0, rig.now), 1.0);
    rig.app.handle_key(Key::Back, rig.now);
    rig.advance(50);
    let half = rig.app.anim.get(Anim::MenuFade, 1.0, rig.now);
    assert!(half > 0.0 && half < 1.0, "{half}");
    assert!(rig.app.context_menu.is_some(), "still fading");
    rig.advance(150);
    rig.advance(0);
    assert!(rig.app.context_menu.is_none(), "gone after the fade");
}

#[test]
fn home_closes_the_menu_and_goes_home() {
    let mut rig = Rig::new();
    open_grid_menu(&mut rig, "com.demo.chrome");
    rig.press(Key::Home);
    assert!(rig.app.context_menu().is_none());
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
}

#[test]
fn add_to_home_from_the_grid_appends_the_app_and_saves() {
    let mut rig = Rig::new();
    let chrome = open_grid_menu(&mut rig, "com.demo.chrome");
    let effects = rig.press(Key::Ok);
    assert_eq!(effects, [Effect::SaveConfig]);
    assert_eq!(rig.app.config.favorites.last().unwrap(), "com.demo.chrome");
    assert!(rig.app.context_menu().is_none());
    assert_eq!(rig.focused(), Some(chrome));
    // The next menu on it offers to remove it.
    rig.press(Key::Menu);
    assert_eq!(menu_actions(&rig)[0], MenuAction::RemoveFromHome);
}

#[test]
fn remove_from_home_from_the_grid_takes_the_favorite_out() {
    let mut rig = Rig::new();
    let youtube = open_grid_menu(&mut rig, YOUTUBE);
    assert_eq!(selected_action(&rig), MenuAction::RemoveFromHome);
    assert_eq!(rig.press(Key::Ok), [Effect::SaveConfig]);
    assert!(!rig.app.config.favorites.iter().any(|p| p == YOUTUBE));
    assert_eq!(rig.focused(), Some(youtube));
}

#[test]
fn with_twelve_favorites_add_is_disabled_and_skipped() {
    let apps: Vec<(String, String)> = (0..13)
        .map(|i| (format!("com.demo.app{i:02}"), format!("App {i:02}")))
        .collect();
    let refs: Vec<(&str, &str)> = apps.iter().map(|(p, l)| (p.as_str(), l.as_str())).collect();
    let mut rig = Rig::with_catalog(catalog(&refs));
    rig.app.config.favorites = apps[..12].iter().map(|(p, _)| p.clone()).collect();
    rig.advance(0);
    open_grid_menu(&mut rig, "com.demo.app12");
    let menu = rig.app.context_menu().unwrap();
    assert_eq!(menu.items[0].action, MenuAction::AddToHome);
    assert!(!menu.items[0].enabled);
    assert_eq!(
        selected_action(&rig),
        MenuAction::AppInfo,
        "the focus starts after it"
    );
    rig.press(Key::Up);
    assert_eq!(
        selected_action(&rig),
        MenuAction::AppInfo,
        "it never takes the focus"
    );
}

#[test]
fn a_system_app_cannot_be_uninstalled() {
    let mut entries: Vec<Discovered> = [
        (NETFLIX, "Netflix"),
        ("com.android.tv.settings", "Configurações"),
    ]
    .iter()
    .map(|(p, l)| discovered(p, l))
    .collect();
    entries[1].system = true;
    let mut rig = Rig::with_catalog(Catalog::from_discovered(&entries, "app.lume.tv"));
    open_grid_menu(&mut rig, "com.android.tv.settings");
    assert_eq!(
        menu_actions(&rig),
        [MenuAction::AddToHome, MenuAction::AppInfo]
    );
}

#[test]
fn app_info_and_uninstall_ask_the_platform() {
    let mut rig = Rig::new();
    open_grid_menu(&mut rig, "com.demo.chrome");
    rig.press(Key::Down);
    assert_eq!(
        rig.press(Key::Ok),
        [Effect::OpenAppInfo {
            package: "com.demo.chrome".into()
        }]
    );
    rig.press(Key::Menu);
    rig.press_all(&[Key::Down, Key::Down]);
    assert_eq!(
        rig.press(Key::Ok),
        [Effect::Uninstall {
            package: "com.demo.chrome".into()
        }]
    );
}

#[test]
fn an_uninstalled_app_leaves_the_focus_on_the_tile_in_its_place() {
    // The grid: Chrome, Disney+, HOME, Jogos, Netflix, Prime Video, Spotify, YouTube.
    let mut rig = Rig::new();
    open_grid_menu(&mut rig, DISNEY);
    rig.press(Key::Back);
    assert_eq!(rig.app.grid_position, Some(1));
    rig.app.catalog.apply(&[], &[DISNEY.to_string()]);
    rig.app.config.remove_uninstalled(&rig.app.catalog);
    rig.app.catalog_changed();
    rig.advance(0);
    assert!(
        !rig.app.config.favorites.iter().any(|p| p == DISNEY),
        "out of the dock"
    );
    assert_eq!(rig.app.grid_position, Some(1));
    assert_eq!(
        rig.focused(),
        Some(ids::GRID_SETTINGS),
        "HOME took its place"
    );
    // The last tile gives the focus to the new last one.
    let youtube = grid_id_of(&rig, YOUTUBE);
    focus_on(&mut rig, youtube);
    assert_eq!(rig.app.grid_position, Some(6));
    rig.app.catalog.apply(&[], &[YOUTUBE.to_string()]);
    rig.app.catalog_changed();
    rig.advance(0);
    assert_eq!(rig.app.grid_position, Some(5));
    assert_eq!(rig.focused(), Some(grid_id_of(&rig, SPOTIFY)));
}

#[test]
fn a_menu_whose_app_is_uninstalled_closes() {
    let mut rig = Rig::new();
    open_grid_menu(&mut rig, "com.demo.chrome");
    rig.app.catalog.apply(&[], &["com.demo.chrome".to_string()]);
    rig.app.catalog_changed();
    assert!(rig.app.context_menu().is_none());
}

#[test]
fn the_dock_menu_has_move_remove_and_app_info() {
    let mut rig = Rig::new();
    open_dock_menu(&mut rig, 0);
    assert_eq!(
        menu_actions(&rig),
        [
            MenuAction::Move,
            MenuAction::RemoveFromHome,
            MenuAction::AppInfo
        ]
    );
}

#[test]
fn only_one_menu_item_is_ever_lit() {
    // The last item of one menu stayed lit in the next one.
    let mut rig = Rig::new();
    let lit = |rig: &Rig| {
        (0..4)
            .filter(|i| rig.app.focus_progress(ids::MENU_BASE + i) > 0.0)
            .count()
    };
    open_grid_menu(&mut rig, "com.demo.chrome");
    rig.press_all(&[Key::Down, Key::Down]);
    rig.advance(300);
    rig.press(Key::Back);
    rig.advance(300);
    rig.press(Key::Menu);
    rig.advance(300);
    assert_eq!(lit(&rig), 1);
    assert_eq!(selected_action(&rig), MenuAction::AddToHome);
    rig.press(Key::Back);
    rig.press(Key::Home);
    rig.advance(300);
    open_dock_menu(&mut rig, 1);
    rig.press_all(&[Key::Down, Key::Down]);
    rig.press(Key::Back);
    rig.advance(300);
    rig.press(Key::Menu);
    rig.advance(300);
    assert_eq!(lit(&rig), 1);
}

/// Opens the menu of the favorite at `index` and starts moving it.
fn start_moving(rig: &mut Rig, index: usize) {
    open_dock_menu(rig, index);
    assert_eq!(selected_action(rig), MenuAction::Move);
    assert!(rig.press(Key::Ok).is_empty(), "nothing is written yet");
    assert!(rig.app.context_menu().is_none());
}

#[test]
fn move_carries_the_favorite_with_the_arrows_and_ok_keeps_the_place() {
    let mut rig = Rig::new();
    let before = rig.app.config.favorites.clone();
    start_moving(&mut rig, 1);
    assert_eq!(rig.app.moving_favorite(), Some(before[1].as_str()));
    assert!(rig.press(Key::Right).is_empty());
    assert!(rig.press(Key::Right).is_empty());
    assert_eq!(rig.app.config.favorites[3], before[1]);
    assert_eq!(
        rig.focused(),
        Some(ids::FAVORITE_BASE + 3),
        "the focus goes with it"
    );
    rig.press_all(&[Key::Up, Key::Down]);
    assert_eq!(
        rig.focused(),
        Some(ids::FAVORITE_BASE + 3),
        "up and down do nothing"
    );
    assert_eq!(rig.press(Key::Ok), [Effect::SaveConfig]);
    assert_eq!(rig.app.moving_favorite(), None);
    let mut expected = before.clone();
    let moved = expected.remove(1);
    expected.insert(3, moved);
    assert_eq!(rig.app.config.favorites, expected);
}

#[test]
fn move_stops_at_the_ends_of_the_dock() {
    let mut rig = Rig::new();
    let before = rig.app.config.favorites.clone();
    start_moving(&mut rig, 0);
    rig.press(Key::Left);
    assert_eq!(rig.app.config.favorites, before);
    rig.press_all(&[Key::Right; 8]);
    assert_eq!(rig.app.config.favorites.last(), Some(&before[0]));
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE + 4));
}

#[test]
fn back_puts_the_moved_favorite_back_where_it_was() {
    let mut rig = Rig::new();
    let before = rig.app.config.favorites.clone();
    start_moving(&mut rig, 2);
    rig.press_all(&[Key::Left, Key::Left]);
    assert_ne!(rig.app.config.favorites, before);
    rig.press(Key::Back);
    assert_eq!(rig.app.config.favorites, before);
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE + 2));
    assert_eq!(rig.app.moving_favorite(), None);
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    // OK without moving writes nothing.
    start_moving(&mut rig, 2);
    assert!(rig.press(Key::Ok).is_empty());
}

#[test]
fn home_while_moving_puts_the_favorite_back_and_goes_home() {
    let mut rig = Rig::new();
    let before = rig.app.config.favorites.clone();
    start_moving(&mut rig, 2);
    rig.press(Key::Right);
    rig.press(Key::Home);
    assert_eq!(rig.app.config.favorites, before);
    assert_eq!(rig.app.moving_favorite(), None);
}

#[test]
fn a_single_favorite_has_no_move() {
    let mut rig = Rig::new();
    rig.app.config.favorites = vec![NETFLIX.to_string()];
    rig.advance(0);
    open_dock_menu(&mut rig, 0);
    assert!(!menu_actions(&rig).contains(&MenuAction::Move));
}

#[test]
fn removing_the_only_favorite_leaves_add_favorites_with_the_focus() {
    let mut rig = Rig::new();
    rig.app.config.favorites = vec![NETFLIX.to_string()];
    rig.advance(0);
    open_dock_menu(&mut rig, 0);
    assert_eq!(
        menu_actions(&rig),
        [MenuAction::RemoveFromHome, MenuAction::AppInfo]
    );
    rig.press(Key::Ok);
    assert!(rig.app.config.favorites.is_empty());
    assert_eq!(rig.focused(), Some(ids::FAVORITES_ADD));
}

#[test]
fn the_editor_shows_what_the_menu_changed() {
    let mut rig = Rig::new();
    open_grid_menu(&mut rig, "com.demo.chrome");
    rig.press(Key::Ok);
    rig.press(Key::Down);
    focus_on(&mut rig, ids::GRID_SETTINGS);
    rig.press_all(&[Key::Ok, Key::Ok]);
    assert_eq!(rig.app.screen(), &Screen::Editor);
    assert!(!rig
        .app
        .addable_apps()
        .iter()
        .any(|(p, _)| p == "com.demo.chrome"));
    rig.press(Key::Menu);
    assert!(rig.app.context_menu().is_none(), "no menu in the editor");
}

#[test]
fn removing_a_favorite_from_the_dock_moves_the_focus_to_its_place() {
    let mut rig = Rig::new();
    let before = rig.app.config.favorites.clone();
    open_dock_menu(&mut rig, 1);
    rig.press(Key::Down);
    assert_eq!(selected_action(&rig), MenuAction::RemoveFromHome);
    assert_eq!(rig.press(Key::Ok), [Effect::SaveConfig]);
    assert_eq!(rig.app.config.favorites.len(), before.len() - 1);
    assert_eq!(
        rig.focused(),
        Some(ids::FAVORITE_BASE + 1),
        "the next one took its place"
    );
    // The last one gives the focus to the one before it.
    open_dock_menu(&mut rig, 3);
    rig.press(Key::Down);
    rig.press(Key::Ok);
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE + 2));
}

#[test]
fn twelve_favorites_with_room_for_the_arrows_fit_the_safe_area() {
    let safe = crate::display::Display::new(1280, 720).safe_area();
    for focus in 0..12 {
        let l = dock::layout_with_room(12, focus as f32, 1.0, 640.0, safe.bottom(), 1.0);
        assert!(
            l.bar.x >= safe.x && l.bar.right() <= safe.right(),
            "focus {focus}: {:?}",
            l.bar
        );
    }
}

// The splash.

use crate::render::{Cmd, RectCmd, RectKind};
use crate::theme::{vars, Paint};

/// lume goes to the background and comes back; the first frame back is drawn.
fn come_back(rig: &mut Rig) -> DisplayList {
    rig.app.went_to_background();
    rig.app.icons.clear();
    rig.app.came_to_foreground();
    rig.frame()
}

/// The splash logo's square in a frame, if it is drawn.
fn splash_logo(list: &DisplayList) -> Option<RectCmd> {
    list.cmds().iter().find_map(|cmd| match cmd {
        // The logo grows from a little under its size while it comes in (and shrinks as it leaves).
        Cmd::Rect(r)
            if (r.rect.w - r.rect.h).abs() < 0.01
                && r.rect.w <= vars::SPLASH_LOGO
                && r.rect.w >= 0.9 * vars::SPLASH_LOGO =>
        {
            Some(*r)
        }
        _ => None,
    })
}

/// The alpha the logo's square is painted with (0 to 255).
fn logo_alpha(list: &DisplayList) -> Option<u8> {
    match splash_logo(list)?.kind {
        RectKind::Fill(Paint::Solid(color)) => Some(color.a),
        _ => None,
    }
}

/// The alpha the wallpaper is drawn with (0 to 255), if it is drawn.
fn wallpaper_alpha(list: &DisplayList) -> Option<u8> {
    list.cmds().iter().find_map(|cmd| match cmd {
        Cmd::Wallpaper(quad) => Some(quad.color.a),
        _ => None,
    })
}

/// The top edge of the dock's background.
fn dock_top(list: &DisplayList) -> f32 {
    list.cmds()
        .iter()
        .find_map(|cmd| match cmd {
            Cmd::Rect(r) if r.radius == 30.0 && matches!(r.kind, RectKind::Fill(_)) => {
                Some(r.rect.y)
            }
            _ => None,
        })
        .expect("the dock is drawn")
}

/// How long the logo is held before the reveal may start (the first wake of a rig is a cold start).
fn hold_ms() -> u64 {
    vars::SPLASH_HOLD_COLD.duration_ms as u64
}

fn load_dock_icons(rig: &mut Rig) {
    for package in rig.app.config.favorites.clone() {
        rig.app.icons.reserve_dock(&package).unwrap();
    }
}

/// Gives every app of the grid its tile image.
fn load_grid_tiles(rig: &mut Rig) {
    for i in 0..rig.app.catalog.apps().len() {
        let package = rig.app.catalog.apps()[i].package.clone();
        rig.app
            .icons
            .reserve(&package, crate::app_icons::Art::Icon)
            .unwrap();
    }
}

#[test]
fn the_splash_has_a_neutral_loading_line_and_no_logo() {
    let mut rig = Rig::new();
    let first = come_back(&mut rig);
    assert!(rig.app.in_splash());
    assert_eq!(logo_alpha(&first), None);
    assert_eq!(first.len(), 1, "{:?}", first.cmds());
    assert!(matches!(
        first.cmds()[0],
        Cmd::Rect(r) if r.rect == Rect::new(600.0, 358.0, 80.0, 4.0)
    ));
}

#[test]
fn the_splash_holds_while_a_dock_icon_is_missing_and_lets_go_when_it_loads() {
    let mut rig = Rig::new();
    let favorites = rig.app.config.favorites.clone();
    assert!(favorites.len() >= 2);
    come_back(&mut rig);
    assert_eq!(rig.app.splash_progress(), 0.0);
    load_grid_tiles(&mut rig);
    for package in &favorites[1..] {
        rig.app.icons.reserve_dock(package).unwrap();
    }
    rig.advance(50);
    assert!(rig.app.in_splash(), "one icon is missing");
    let partial = rig.app.splash_progress();
    assert!(partial > 0.0 && partial < 1.0, "{partial}");
    rig.app.icons.reserve_dock(&favorites[0]).unwrap();
    rig.advance(hold_ms());
    assert!(
        !rig.app.in_splash(),
        "all icons are there: the reveal starts"
    );
    assert_eq!(rig.app.splash_progress(), 1.0);
}

#[test]
fn an_icon_that_failed_counts_as_ready() {
    let mut rig = Rig::new();
    let favorites = rig.app.config.favorites.clone();
    come_back(&mut rig);
    load_grid_tiles(&mut rig);
    for package in &favorites[1..] {
        rig.app.icons.reserve_dock(package).unwrap();
    }
    rig.app.icon_failed(&favorites[0], true);
    rig.advance(hold_ms());
    assert!(!rig.app.in_splash());
}

#[test]
fn the_splash_lets_go_after_600_ms_without_asking_for_frames_meanwhile() {
    let mut rig = Rig::new();
    come_back(&mut rig);
    let since = rig.now;
    // Once the logo is in nothing moves while it waits: the next frame is the 600 ms deadline (the
    // clock's minute is further away), so the loop sleeps and keeps loading icons.
    rig.advance(vars::SPLASH_LOGO_IN.duration_ms as u64);
    assert!(!rig.app.is_animating(rig.now));
    assert_eq!(rig.app.next_deadline(rig.now), Some(since + 600));
    rig.advance(599 - vars::SPLASH_LOGO_IN.duration_ms as u64);
    assert!(rig.app.in_splash());
    rig.advance(1);
    assert!(!rig.app.in_splash(), "the wait has a limit");
}

#[test]
fn the_splash_waits_for_the_visible_tiles_of_the_grid_it_returns_to() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    rig.press(Key::Down);
    let focus = rig.focused();
    rig.press(Key::Ok);
    come_back(&mut rig);
    rig.press(Key::Home);
    assert!(
        rig.app.in_splash(),
        "the Home press that brings lume back keeps the splash"
    );
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Apps));
    assert_eq!(
        rig.focused(),
        focus,
        "the focus is restored behind the splash"
    );
    let missing = rig.app.splash_missing_tiles().to_vec();
    assert_eq!(
        missing.len(),
        rig.app.catalog.apps().len(),
        "all tiles fit the view"
    );
    for package in &missing {
        rig.app
            .icons
            .reserve(package, crate::app_icons::Art::Icon)
            .unwrap();
    }
    rig.advance(10);
    assert!(
        rig.app.in_splash(),
        "the dock icons, for a move to Home, are missing"
    );
    load_dock_icons(&mut rig);
    rig.advance(hold_ms());
    assert!(!rig.app.in_splash());
}

#[test]
fn the_home_splash_also_waits_for_the_first_screen_of_the_grid() {
    let mut rig = Rig::new();
    come_back(&mut rig);
    load_dock_icons(&mut rig);
    rig.advance(10);
    // A move to the Apps tab right after the reveal must not show tiles arriving.
    assert!(rig.app.in_splash());
    assert_eq!(
        rig.app.splash_missing_tiles().len(),
        rig.app.catalog.apps().len()
    );
    load_grid_tiles(&mut rig);
    rig.advance(hold_ms());
    assert!(!rig.app.in_splash());
}

#[test]
fn a_key_during_the_splash_ends_it_and_is_applied() {
    let mut rig = Rig::new();
    let first = rig.focused();
    come_back(&mut rig);
    rig.press(Key::Right);
    assert!(!rig.app.in_splash());
    assert!(
        rig.app.take_interface_ready(),
        "the interface is ready at once"
    );
    assert_ne!(rig.focused(), first, "the focus moved to the right");
    let list = rig.frame();
    assert_eq!(logo_alpha(&list), None, "no reveal: the logo is gone");
    assert_eq!(wallpaper_alpha(&list), Some(255));
}

#[test]
fn home_during_the_splash_follows_the_usual_rules() {
    let mut rig = Rig::new();
    open_apps(&mut rig);
    come_back(&mut rig);
    // Not back from an app opened by lume: Home goes to the start, behind the splash.
    rig.press(Key::Home);
    assert!(rig.app.in_splash());
    assert_eq!(rig.app.screen(), &Screen::Main(Tab::Home));
    assert_eq!(rig.focused(), Some(ids::FAVORITE_BASE));
}

#[test]
fn the_reveal_brings_up_the_wallpaper_and_the_content_without_branding() {
    let mut rig = Rig::new();
    let rest = dock_top(&rig.frame());
    for (cold, duration) in [
        (true, vars::SPLASH_REVEAL_COLD.duration_ms),
        (false, vars::SPLASH_REVEAL_WARM.duration_ms),
    ] {
        come_back(&mut rig);
        rig.advance(vars::SPLASH_LOGO_IN.duration_ms as u64);
        load_dock_icons(&mut rig);
        load_grid_tiles(&mut rig);
        // The logo is held before the reveal starts (the entry already took part of it).
        let hold = if cold {
            vars::SPLASH_HOLD_COLD
        } else {
            vars::SPLASH_HOLD_WARM
        };
        rig.now += hold.duration_ms as u64 - vars::SPLASH_LOGO_IN.duration_ms as u64;
        let start = rig.frame();
        assert!(!rig.app.in_splash(), "cold: {cold}");
        assert_eq!(logo_alpha(&start), None);
        assert_eq!(
            wallpaper_alpha(&start),
            None,
            "intensity 0: only the background"
        );

        rig.now += (duration / 2.0) as u64;
        let middle = rig.frame();
        // Linear progress 0.5 through the overlapping stages: the interface is partly there.
        let stage = |from: f32, to: f32| {
            crate::ui::animation::ease(crate::theme::Curve::EaseInOut, (0.5 - from) / (to - from))
        };
        let alpha = |v: f32| (255.0 * v).round() as i32;
        assert!((i32::from(wallpaper_alpha(&middle).unwrap()) - alpha(stage(0.0, 0.8))).abs() <= 1);
        assert_eq!(logo_alpha(&middle), None);
        assert!((dock_top(&middle) - rest - 16.0 * (1.0 - stage(0.3, 1.0))).abs() < 0.01);
        assert!(!rig.app.take_interface_ready());

        rig.now += duration as u64;
        let end = rig.frame();
        assert_eq!(logo_alpha(&end), None);
        assert_eq!(wallpaper_alpha(&end), Some(255));
        assert_eq!(dock_top(&end), rest);
        assert!(rig.app.take_interface_ready(), "the end of the reveal");
        assert!(!rig.app.is_animating(rig.now));
    }
}

#[test]
fn the_logo_fades_only_where_it_is() {
    let mut rig = Rig::new();
    come_back(&mut rig);
    load_dock_icons(&mut rig);
    rig.frame();
    rig.now += 100;
    let list = rig.frame();
    // Only the logo's four shapes are translucent paint over the whole frame: no full-screen layer.
    let full_screen = list
        .cmds()
        .iter()
        .any(|cmd| matches!(cmd, Cmd::Rect(r) if r.rect.w >= 1280.0 && r.rect.h >= 720.0));
    assert!(!full_screen);
}

/// The alpha of the dock icon images in a frame (dock cells are the square images).
fn dock_image_alphas(list: &DisplayList) -> Vec<u8> {
    list.cmds()
        .iter()
        .filter_map(|cmd| match cmd {
            Cmd::Image(t) if (t.dest.w - t.dest.h).abs() < 0.01 => Some(t.color.a),
            _ => None,
        })
        .collect()
}

/// The key mapping screen as the tests below drive it.
mod keymap {
    use super::*;
    use crate::app::context_menu::{MenuAction, Origin};
    use crate::app::keymap::*;
    use crate::app::keymap::{CONFIRM_CANCEL, CONFIRM_RESTORE};
    use crate::keys::{CommandTarget, KeyTarget, SettingShortcut};

    impl Rig {
        /// A key of the remote control, by its Android code.
        fn code(&mut self, code: i32) -> Vec<Effect> {
            let effects = self.app.handle_keycode(code, self.now);
            self.now += 100;
            self.frame();
            effects
        }

        fn menu_actions(&self) -> Vec<(MenuAction, bool)> {
            self.app
                .context_menu()
                .map(|m| m.items.iter().map(|i| (i.action, i.enabled)).collect())
                .unwrap_or_default()
        }
    }

    fn menu() -> KeyTarget {
        KeyTarget::Command(CommandTarget::Menu)
    }

    fn opened() -> Rig {
        let mut rig = Rig::new();
        open_key_map(&mut rig);
        rig
    }

    /// On the row `id`, with its menu open.
    fn open_menu_of(rig: &mut Rig, id: FocusId) {
        assert!(rig.app.focus_for_test(id), "{id} is on the screen");
        rig.press(Key::Ok);
        assert!(rig.app.context_menu().is_some(), "the menu of {id}");
    }

    /// Chooses the item `action` of the open menu.
    fn choose(rig: &mut Rig, action: MenuAction) {
        let index = rig
            .menu_actions()
            .iter()
            .position(|(a, _)| *a == action)
            .unwrap_or_else(|| panic!("{action:?} is not in the menu"));
        let selected = rig.app.context_menu().unwrap().selected;
        for _ in selected..index {
            rig.press(Key::Down);
        }
        for _ in index..selected {
            rig.press(Key::Up);
        }
        rig.press(Key::Ok);
    }

    #[test]
    fn the_screen_starts_on_the_first_command_and_has_no_horizontal_focus() {
        let mut rig = opened();
        assert_eq!(rig.focused(), Some(CMD_BASE));
        rig.press_all(&[Key::Right, Key::Left, Key::Right]);
        assert_eq!(rig.focused(), Some(CMD_BASE), "no row to the side");
    }

    #[test]
    fn down_visits_every_line_in_order() {
        let mut rig = opened();
        let mut seen = vec![rig.focused().unwrap()];
        for _ in 0..8 {
            rig.press(Key::Down);
            let id = rig.focused().unwrap();
            if seen.last() != Some(&id) {
                seen.push(id);
            }
        }
        assert_eq!(
            seen,
            [
                CMD_BASE,
                CMD_BASE + 1,
                ADD_SETTING,
                ADD_APP,
                RESTORE_FACTORY
            ]
        );
    }

    #[test]
    fn the_first_rows_are_where_the_design_puts_them() {
        let mut rig = opened();
        let rect = |rig: &mut Rig, id| {
            assert!(rig.app.focus_for_test(id));
            rig.app.focus.current().unwrap().rect
        };
        // Panel 1 starts at 212 (where the content of a secondary screen starts), 216 tall.
        assert_eq!(
            rect(&mut rig, CMD_BASE),
            Rect::new(96.0, 268.0, 1088.0, 64.0)
        );
        assert_eq!(
            rect(&mut rig, CMD_BASE + 1),
            Rect::new(96.0, 340.0, 1088.0, 64.0)
        );
        // Panel 2 starts at 452 and has only the "add" row; panel 3 at 620.
        assert_eq!(
            rect(&mut rig, ADD_SETTING),
            Rect::new(96.0, 508.0, 1088.0, 64.0)
        );
        assert_eq!(
            rect(&mut rig, ADD_APP),
            Rect::new(96.0, 676.0, 1088.0, 64.0)
        );
        let restore = rect(&mut rig, RESTORE_FACTORY);
        assert_eq!((restore.x, restore.y, restore.h), (64.0, 788.0, 48.0));
        assert_eq!(
            restore.bottom(),
            836.0,
            "the page ends at 836 without shortcuts"
        );
    }

    #[test]
    fn a_panel_grows_by_72_per_shortcut_and_the_add_row_goes_when_all_are_mapped() {
        let mut rig = opened();
        let count = SettingShortcut::COUNT;
        for (i, setting) in SettingShortcut::ALL.into_iter().enumerate() {
            rig.app
                .keys
                .add_code(&KeyTarget::Setting(setting), 200 + i as i32);
        }
        rig.frame();
        assert!(!rig.app.focus_for_test(ADD_SETTING), "nothing left to add");
        // Panel 2: 72 + 72 x 20 tall, from 452; panel 3 starts 24 below it.
        assert!(rig.app.focus_for_test(SETTING_BASE + count as FocusId - 1));
        assert_eq!(
            rig.app.focus.current().unwrap().rect.y,
            452.0 + 56.0 + (count as f32 - 1.0) * 72.0
        );
        let panel_2_bottom = 452.0 + 72.0 + 72.0 * count as f32;
        assert!(rig.app.focus_for_test(ADD_APP));
        assert_eq!(
            rig.app.focus.current().unwrap().rect.y,
            panel_2_bottom + 24.0 + 56.0
        );
    }

    #[test]
    fn the_scrolling_follows_the_focus_down_to_the_last_button() {
        let mut rig = opened();
        for _ in 0..4 {
            rig.press(Key::Down);
        }
        assert_eq!(rig.focused(), Some(RESTORE_FACTORY));
        rig.advance(1000);
        assert!(rig.app.scroll_y > 0.0, "the page scrolled");
        let restore = rig.app.focus.current().unwrap().rect;
        let on_screen = restore.translate(0.0, -rig.app.scroll_y);
        assert!(
            on_screen.bottom() <= 720.0,
            "the button is in view: {on_screen:?}"
        );
        for _ in 0..4 {
            rig.press(Key::Up);
        }
        assert_eq!(rig.focused(), Some(CMD_BASE));
        rig.advance(1000);
        assert_eq!(
            rig.app.scroll_y, 0.0,
            "the first row brings the header back"
        );
    }

    #[test]
    fn ok_on_a_command_opens_its_menu_with_a_disabled_restore_while_at_the_default() {
        let mut rig = opened();
        open_menu_of(&mut rig, CMD_BASE);
        assert_eq!(
            rig.menu_actions(),
            [
                (MenuAction::AddKey, true),
                (MenuAction::RemoveKey(82), true),
                (MenuAction::RemoveKey(174), true),
                (MenuAction::RestoreDefault, false),
            ]
        );
        assert_eq!(
            rig.app.context_menu().unwrap().origin,
            Origin::KeyRow(CMD_BASE)
        );
        // Back closes it and the focus stays on the row.
        rig.press(Key::Back);
        rig.advance(1000);
        assert!(rig.app.context_menu().is_none());
        assert_eq!(rig.focused(), Some(CMD_BASE));
    }

    #[test]
    fn removing_one_key_of_a_command_saves_and_enables_the_restore() {
        let mut rig = opened();
        open_menu_of(&mut rig, CMD_BASE);
        // The first enabled item has the focus: "Add key". Down twice reaches "Remove 174".
        rig.press_all(&[Key::Down, Key::Down]);
        let effects = rig.press(Key::Ok);
        assert_eq!(effects, vec![Effect::SaveKeys]);
        assert_eq!(rig.app.keys.codes(Key::Menu), &[82]);
        assert_eq!(rig.focused(), Some(CMD_BASE), "the focus stays on the row");
        open_menu_of(&mut rig, CMD_BASE);
        assert_eq!(
            rig.menu_actions().last(),
            Some(&(MenuAction::RestoreDefault, true))
        );
        choose(&mut rig, MenuAction::RestoreDefault);
        assert_eq!(rig.app.keys.codes(Key::Menu), &[82, 174]);
    }

    #[test]
    fn a_disabled_restore_never_takes_the_focus() {
        let mut rig = opened();
        open_menu_of(&mut rig, CMD_BASE + 1);
        for _ in 0..6 {
            rig.press(Key::Down);
        }
        let selected = rig.app.context_menu().unwrap().selected;
        assert!(rig.app.context_menu().unwrap().items[selected].enabled);
        assert_eq!(
            rig.menu_actions().last(),
            Some(&(MenuAction::RestoreDefault, false))
        );
    }

    #[test]
    fn adding_a_key_listens_records_it_and_moves_it_from_the_other_command() {
        let mut rig = opened();
        open_menu_of(&mut rig, CMD_BASE);
        choose(&mut rig, MenuAction::AddKey);
        assert!(rig.app.is_listening_key());
        assert_eq!(rig.app.keymap.listening, Some(menu()));
        assert_eq!(rig.focused(), None, "the card takes no focus");

        let effects = rig.code(100);
        assert_eq!(effects, vec![Effect::SaveKeys]);
        assert!(!rig.app.is_listening_key());
        assert!(rig.app.keys.codes(Key::Menu).contains(&100));
        assert!(rig.app.toast_text().is_some(), "the key was assigned");
        assert_eq!(
            rig.focused(),
            Some(CMD_BASE),
            "back on the row it came from"
        );

        // The same key given to the other command leaves the first.
        open_menu_of(&mut rig, CMD_BASE + 1);
        choose(&mut rig, MenuAction::AddKey);
        rig.code(100);
        assert!(rig.app.keys.inputs().contains(&100));
        assert!(!rig.app.keys.codes(Key::Menu).contains(&100));
    }

    #[test]
    fn a_command_without_keys_goes_straight_to_the_listening() {
        let mut rig = opened();
        rig.app
            .keys
            .remove_code(&KeyTarget::Command(CommandTarget::Inputs), 178);
        rig.frame();
        assert!(rig.app.focus_for_test(CMD_BASE + 1));
        rig.press(Key::Ok);
        assert!(rig.app.context_menu().is_none(), "no menu to choose from");
        assert!(rig.app.is_listening_key());
    }

    #[test]
    fn only_back_cancels_the_listening_and_reserved_keys_are_refused() {
        let mut rig = opened();
        open_menu_of(&mut rig, CMD_BASE);
        choose(&mut rig, MenuAction::AddKey);
        assert!(!rig.app.keymap.reserved_warning);
        let before = rig.app.keys.clone();
        // Arrows, OK (all four codes), Home and the power keys: the warning, nothing recorded.
        for code in [19, 20, 21, 22, 23, 66, 160, 96, 3, 26, 177, 223, 224, 276] {
            let effects = rig.code(code);
            assert!(effects.is_empty(), "{code}");
            assert!(rig.app.is_listening_key(), "{code} does not cancel");
            assert!(rig.app.keymap.reserved_warning, "{code} warns");
        }
        assert!(rig.app.keys == before, "nothing was recorded");
        // Back cancels without changing the mapping, and the focus goes back to the row.
        for code in [4, 111, 97] {
            open_menu_of_after_cancel(&mut rig, code);
        }
    }

    /// Back with the code `code` cancels a listening started again from the row.
    fn open_menu_of_after_cancel(rig: &mut Rig, code: i32) {
        if !rig.app.is_listening_key() {
            open_menu_of(rig, CMD_BASE);
            choose(rig, MenuAction::AddKey);
        }
        let before = rig.app.keys.clone();
        let effects = rig.code(code);
        assert!(effects.is_empty());
        assert!(!rig.app.is_listening_key());
        assert!(!rig.app.keymap.reserved_warning);
        assert!(rig.app.keys == before);
        assert_eq!(rig.focused(), Some(CMD_BASE));
    }

    #[test]
    fn the_logical_keys_of_the_desktop_viewer_listen_the_same_way() {
        let mut rig = opened();
        open_menu_of(&mut rig, CMD_BASE);
        choose(&mut rig, MenuAction::AddKey);
        for key in [
            Key::Up,
            Key::Down,
            Key::Left,
            Key::Right,
            Key::Ok,
            Key::Home,
        ] {
            rig.press(key);
            assert!(rig.app.is_listening_key(), "{key:?}");
            assert!(rig.app.keymap.reserved_warning, "{key:?}");
        }
        rig.press(Key::Back);
        assert!(!rig.app.is_listening_key());
        assert_eq!(rig.app.screen(), &Screen::KeyMap, "Back only cancelled");
    }

    #[test]
    fn the_listening_card_is_drawn_without_a_dark_layer_and_the_focus_tree_may_be_empty() {
        let mut rig = opened();
        open_menu_of(&mut rig, CMD_BASE);
        choose(&mut rig, MenuAction::AddKey);
        let list = rig.frame();
        let visible = rig.display.visible_rect();
        let layer = list.cmds().iter().any(|cmd| {
            matches!(cmd, crate::render::Cmd::Rect(r)
                if r.rect.w >= visible.w - 1.0 && r.rect.h >= visible.h - 1.0)
        });
        assert!(!layer, "no full-screen layer over the screen");
        assert!(rig.app.focus.rows().is_empty());
        assert_eq!(rig.focused(), None);
    }

    /// The "Add shortcut" row, opened.
    fn open_pick_setting(rig: &mut Rig) {
        assert!(rig.app.focus_for_test(ADD_SETTING));
        rig.press(Key::Ok);
        assert_eq!(rig.app.screen(), &Screen::KeyMapPickSetting);
    }

    #[test]
    fn every_settings_shortcut_is_in_the_choose_a_shortcut_list_and_cancelling_creates_nothing() {
        let mut rig = opened();
        open_pick_setting(&mut rig);
        assert_eq!(rig.focused(), Some(PICK_SETTING_BASE));
        let count = SettingShortcut::COUNT;
        assert_eq!(rig.app.pick_setting_rows().len(), count);
        // Every one can be reached with Down, and the list scrolls to show it.
        let mut reached = vec![rig.focused().unwrap()];
        for _ in 0..count + 2 {
            rig.press(Key::Down);
            reached.push(rig.focused().unwrap());
            rig.advance(400);
            let rect = rig
                .app
                .focus
                .current()
                .unwrap()
                .rect
                .translate(0.0, -rig.app.scroll_y);
            assert!(rect.y >= 0.0 && rect.bottom() <= 720.0, "in view: {rect:?}");
        }
        reached.dedup();
        assert_eq!(
            reached,
            (0..count as FocusId)
                .map(|i| PICK_SETTING_BASE + i)
                .collect::<Vec<_>>()
        );

        // Choosing one listens for its key; cancelling creates nothing.
        assert!(rig.app.focus_for_test(PICK_SETTING_BASE));
        rig.press(Key::Ok);
        assert_eq!(rig.app.screen(), &Screen::KeyMap);
        assert_eq!(
            rig.app.keymap.listening,
            Some(KeyTarget::Setting(SettingShortcut::Sleep))
        );
        assert!(
            rig.app.keys.setting_shortcuts().is_empty(),
            "not created yet"
        );
        rig.code(4);
        assert!(rig.app.keys.setting_shortcuts().is_empty());
        assert_eq!(rig.focused(), Some(ADD_SETTING), "back on the add row");
    }

    #[test]
    fn back_from_the_choose_a_shortcut_list_returns_to_the_add_row() {
        let mut rig = opened();
        open_pick_setting(&mut rig);
        rig.press(Key::Back);
        assert_eq!(rig.app.screen(), &Screen::KeyMap);
        assert_eq!(rig.focused(), Some(ADD_SETTING));
        assert!(rig.app.keys.setting_shortcuts().is_empty());
    }

    #[test]
    fn a_shortcut_with_a_key_leaves_the_choose_a_shortcut_list() {
        let mut rig = opened();
        rig.app
            .keys
            .add_code(&KeyTarget::Setting(SettingShortcut::Sleep), 183);
        rig.frame();
        open_pick_setting(&mut rig);
        let listed: Vec<_> = rig
            .app
            .pick_setting_rows()
            .into_iter()
            .map(|(_, s, _)| s)
            .collect();
        assert_eq!(listed.len(), SettingShortcut::COUNT - 1);
        assert!(!listed.contains(&SettingShortcut::Sleep));
        assert_eq!(listed[0], SettingShortcut::Picture);
    }

    #[test]
    fn a_settings_shortcut_exists_once_its_key_is_recorded_and_can_be_removed() {
        let mut rig = opened();
        open_pick_setting(&mut rig);
        rig.press(Key::Ok);
        let effects = rig.code(174);
        assert_eq!(effects, vec![Effect::SaveKeys]);
        assert_eq!(
            rig.app
                .keys
                .codes_for(&KeyTarget::Setting(SettingShortcut::Sleep)),
            &[174]
        );
        assert_eq!(rig.focused(), Some(SETTING_BASE), "on the new row");
        // 174 moved off the context menu.
        assert!(!rig.app.keys.codes(Key::Menu).contains(&174));

        open_menu_of(&mut rig, SETTING_BASE);
        assert_eq!(
            rig.menu_actions(),
            [
                (MenuAction::AddKey, true),
                (MenuAction::RemoveKey(174), true),
                (MenuAction::RemoveShortcut, true),
            ]
        );
        choose(&mut rig, MenuAction::RemoveShortcut);
        assert!(rig.app.keys.setting_shortcuts().is_empty());
        assert_eq!(
            rig.focused(),
            Some(ADD_SETTING),
            "the add row took its place"
        );
        // The shortcut is in the list again, first.
        open_pick_setting(&mut rig);
        assert_eq!(rig.app.pick_setting_rows()[0].1, SettingShortcut::Sleep);
    }

    #[test]
    fn removing_the_only_key_of_a_shortcut_removes_the_row() {
        let mut rig = opened();
        rig.app
            .keys
            .add_code(&KeyTarget::Setting(SettingShortcut::Wifi), 183);
        rig.app
            .keys
            .add_code(&KeyTarget::Setting(SettingShortcut::Sound), 184);
        rig.frame();
        open_menu_of(&mut rig, SETTING_BASE);
        choose(&mut rig, MenuAction::RemoveKey(183));
        assert_eq!(rig.app.keys.setting_shortcuts().len(), 1);
        assert_eq!(
            rig.focused(),
            Some(SETTING_BASE),
            "the next row took its place"
        );
    }

    #[test]
    fn the_choose_an_app_screen_lists_every_app_and_back_returns_to_the_add_row() {
        let mut rig = opened();
        rig.app.focus_for_test(ADD_APP);
        rig.press(Key::Ok);
        assert_eq!(rig.app.screen(), &Screen::KeyMapPickApp);
        assert_eq!(rig.focused(), Some(PICK_BASE));
        // The seven apps of the catalog, in the alphabetical order of the grid.
        let apps = rig.app.catalog.apps().len();
        assert_eq!(apps, 7);
        let mut reached = vec![rig.focused().unwrap()];
        for _ in 0..4 {
            rig.press(Key::Right);
            rig.press(Key::Down);
            reached.push(rig.focused().unwrap());
        }
        assert!(
            rig.app.focus_for_test(PICK_BASE + 6),
            "the seventh app has a tile"
        );
        let first = rig.app.focus.rows()[0].items[0].rect;
        assert_eq!(
            (first.x, first.y),
            (64.0, 212.0),
            "the grid starts at y = 212"
        );

        rig.press(Key::Back);
        assert_eq!(rig.app.screen(), &Screen::KeyMap);
        assert_eq!(rig.focused(), Some(ADD_APP));
        assert!(
            rig.app.keys.app_shortcuts().is_empty(),
            "nothing was created"
        );
    }

    #[test]
    fn choosing_an_app_listens_for_its_key_and_the_app_leaves_the_choice() {
        let mut rig = opened();
        rig.app.focus_for_test(ADD_APP);
        rig.press_all(&[Key::Ok, Key::Right]);
        let chosen = rig.app.catalog.apps()[1].package.clone();
        rig.press(Key::Ok);
        assert_eq!(rig.app.screen(), &Screen::KeyMap);
        assert_eq!(
            rig.app.keymap.listening,
            Some(KeyTarget::App(chosen.clone()))
        );
        // Cancelling comes back to the add row with nothing created.
        rig.code(4);
        assert_eq!(rig.focused(), Some(ADD_APP));
        assert!(!rig.app.keys.has_app(&chosen));

        rig.press_all(&[Key::Ok, Key::Right, Key::Ok]);
        let effects = rig.code(183);
        assert_eq!(effects, vec![Effect::SaveKeys]);
        assert_eq!(
            rig.app.keys.codes_for(&KeyTarget::App(chosen.clone())),
            &[183]
        );
        assert_eq!(rig.focused(), Some(APP_BASE), "on the new row");
        // Pressing the key opens the app.
        let effects = rig.code(183);
        assert!(matches!(&effects[..], [Effect::OpenApp { package, .. }] if *package == chosen));
        // The app is not offered again.
        rig.app.focus_for_test(ADD_APP);
        rig.app.go_home();
        rig.app.open_keymap();
        rig.frame();
        rig.app.focus_for_test(ADD_APP);
        rig.press(Key::Ok);
        let offered: Vec<_> = rig
            .app
            .pick_cells()
            .into_iter()
            .map(|c| c.package)
            .collect();
        assert_eq!(offered.len(), 6);
        assert!(!offered.contains(&chosen));
    }

    #[test]
    fn many_apps_all_get_a_tile_and_the_grid_scrolls_to_the_last() {
        let apps: Vec<(String, String)> = (0..40)
            .map(|i| (format!("com.demo.app{i:02}"), format!("App {i:02}")))
            .collect();
        let refs: Vec<(&str, &str)> = apps.iter().map(|(p, l)| (p.as_str(), l.as_str())).collect();
        let mut rig = Rig::with_catalog(catalog(&refs));
        open_key_map(&mut rig);
        rig.app.focus_for_test(ADD_APP);
        rig.press(Key::Ok);
        assert_eq!(rig.app.screen(), &Screen::KeyMapPickApp);
        for index in 0..40 {
            assert!(
                rig.app.focus_for_test(PICK_BASE + index),
                "app {index} has a tile"
            );
        }
        rig.advance(1000);
        assert!(rig.app.scroll_y > 0.0);
        let rect = rig
            .app
            .focus
            .current()
            .unwrap()
            .rect
            .translate(0.0, -rig.app.scroll_y);
        assert!(rect.y >= 0.0 && rect.bottom() <= 720.0, "{rect:?}");
    }

    #[test]
    fn the_factory_dialog_starts_on_cancel_and_back_closes_it() {
        let mut rig = opened();
        rig.app.keys.add_code(&menu(), 999);
        rig.app.focus_for_test(RESTORE_FACTORY);
        rig.press(Key::Ok);
        assert!(rig.app.keymap.confirm_restore);
        assert_eq!(rig.focused(), Some(CONFIRM_CANCEL));
        // The dialog traps the focus: Left goes to "Restore", never to the page.
        rig.press(Key::Left);
        assert_eq!(rig.focused(), Some(CONFIRM_RESTORE));
        rig.press(Key::Up);
        assert_eq!(rig.focused(), Some(CONFIRM_RESTORE));
        rig.press(Key::Back);
        assert!(!rig.app.keymap.confirm_restore);
        assert_eq!(rig.focused(), Some(RESTORE_FACTORY));
        assert!(
            rig.app.keys.codes(Key::Menu).contains(&999),
            "nothing changed"
        );
    }

    #[test]
    fn confirming_the_factory_dialog_restores_commands_and_clears_the_shortcuts() {
        let mut rig = opened();
        rig.app.keys.add_code(&menu(), 999);
        rig.app
            .keys
            .add_code(&KeyTarget::Setting(SettingShortcut::Wifi), 183);
        rig.app
            .keys
            .add_code(&KeyTarget::App(NETFLIX.to_string()), 184);
        rig.app.focus_for_test(RESTORE_FACTORY);
        rig.press(Key::Ok);
        rig.press(Key::Left);
        let effects = rig.press(Key::Ok);
        assert_eq!(effects, vec![Effect::SaveKeys]);
        assert!(!rig.app.keymap.confirm_restore);
        assert!(rig.app.keys.is_all_default());
        assert!(rig.app.keys.setting_shortcuts().is_empty());
        assert!(rig.app.keys.app_shortcuts().is_empty());
        assert_eq!(rig.app.keys.inputs(), &[178]);
    }

    #[test]
    fn the_dialog_is_a_light_card_of_560_by_260_without_a_dark_layer() {
        let mut rig = opened();
        rig.app.focus_for_test(RESTORE_FACTORY);
        rig.press(Key::Ok);
        let list = rig.frame();
        let visible = rig.display.visible_rect();
        let full_screen = list.cmds().iter().any(|cmd| {
            matches!(cmd, crate::render::Cmd::Rect(r)
                if r.rect.w >= visible.w - 1.0 && r.rect.h >= visible.h - 1.0)
        });
        assert!(!full_screen);
        // The card, and its shadow a few units lower.
        let card = list.cmds().iter().find_map(|cmd| match cmd {
            crate::render::Cmd::Rect(r)
                if (r.rect.w - 560.0).abs() < 0.5 && (r.rect.y - 232.0).abs() < 0.5 =>
            {
                Some(r.rect)
            }
            _ => None,
        });
        let card = card.expect("a 560 wide card at y = 232");
        assert_eq!((card.x, card.y, card.h), (360.0, 232.0, 260.0));
        let (restore, cancel) = {
            let row = &rig.app.focus.rows()[0];
            (row.items[0].rect, row.items[1].rect)
        };
        assert_eq!((restore.h, cancel.h), (48.0, 48.0));
        assert_eq!(cancel.y, 232.0 + 180.0);
        assert_eq!(cancel.right(), 888.0, "right-aligned inside the padding");
        assert_eq!(cancel.x - restore.right(), 16.0, "16 between the buttons");
    }

    #[test]
    fn a_shortcut_that_leaves_lume_is_something_to_return_from() {
        let mut rig = Rig::new();
        rig.app
            .keys
            .add_code(&KeyTarget::App(NETFLIX.to_string()), 203);
        rig.advance(1000);
        let effects = rig.code(203);
        assert!(matches!(&effects[..], [Effect::OpenApp { .. }]));
        assert!(rig.app.return_point.is_some());
    }
}

#[test]
fn handle_keycode_triggers_shortcuts_for_commands_settings_and_apps() {
    let mut rig = Rig::new();

    // Custom mapping: 200 -> Inputs command
    rig.app.keys.add_code(
        &crate::keys::KeyTarget::Command(crate::keys::CommandTarget::Inputs),
        200,
    );
    let effects = rig.app.handle_keycode(200, rig.now);
    assert_eq!(effects, vec![Effect::OpenInputs]);

    // Custom mapping: 201 -> Picture setting
    rig.app.keys.add_code(
        &crate::keys::KeyTarget::Setting(crate::keys::SettingShortcut::Picture),
        201,
    );
    let effects = rig.app.handle_keycode(201, rig.now);
    assert_eq!(
        effects,
        vec![Effect::OpenTvSetting {
            setting: crate::keys::SettingShortcut::Picture
        }]
    );

    // Custom mapping: 202 -> Lume Settings
    rig.app.keys.add_code(
        &crate::keys::KeyTarget::Setting(crate::keys::SettingShortcut::LumeSettings),
        202,
    );
    let effects = rig.app.handle_keycode(202, rig.now);
    assert!(effects.is_empty());
    assert_eq!(rig.app.screen(), &Screen::Settings);

    // Custom mapping: 203 -> Netflix app
    rig.app
        .keys
        .add_code(&crate::keys::KeyTarget::App(NETFLIX.to_string()), 203);
    let effects = rig.app.handle_keycode(203, rig.now);
    assert_eq!(
        effects,
        vec![Effect::OpenApp {
            package: NETFLIX.to_string(),
            activity: format!("{NETFLIX}.Main"),
        }]
    );

    // Pure navigation keys still work natively
    rig.app.go_home();
    let effects = rig.app.handle_keycode(20, rig.now); // KEYCODE_DPAD_DOWN
    assert!(effects.is_empty());
}
