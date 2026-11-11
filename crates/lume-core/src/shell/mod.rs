//! The part of lume that sits between the launcher ([`LumeApp`]) and the platform.
//!
//! [`Shell`] owns the launcher and everything it needs to run on a device: the files that
//! persist the user's data, the catalog changes reported by the system, and the memory
//! lifecycle. The Android layer only has to feed it events and draw the frames it builds.
//!
//! Lifecycle: **active** (the window is on screen: the text engine and its glyph atlas exist)
//! and **dormant** (another app is in front: only the small data lives on, a few kilobytes).

pub use crate::keys;
pub mod protocol;

use crate::app::{Effect, Key, LumeApp, Painter};
use crate::catalog::{Catalog, Discovered};
use crate::clock::WallClock;
use crate::display::{self, Display};
use crate::home::config::HomeConfig;
use crate::i18n::Translator;
use crate::render::{DisplayList, Frame, Viewport};
use crate::text::{SystemFonts, TextEngine};
use crate::usage::UsageHistory;
use std::path::{Path, PathBuf};
use std::time::Duration;

const CONFIG_FILE: &str = "config.v1";
const CATALOG_FILE: &str = "catalog.v1";
const USAGE_FILE: &str = "usage.v1";
const KEYS_FILE: &str = "keys.v1";

/// How many times a frame is built again when the glyph atlas was rebuilt in the middle of it.
const MAX_FRAME_PASSES: usize = 4;

pub struct Shell {
    app: LumeApp,
    files: PathBuf,
    own_package: String,
    /// Exists only while active.
    text: Option<TextEngine>,
    display: Option<Display>,
    list: DisplayList,
    /// Something changed that has not been drawn yet.
    dirty: bool,
}

impl Shell {
    /// Loads the saved data and builds the launcher.
    ///
    /// `discover` is called only when there is no saved catalog (the first run), and returns
    /// every launchable app of the device.
    pub fn start(
        files: &Path,
        own_package: &str,
        locales: &[String],
        clock: WallClock,
        discover: &mut dyn FnMut() -> Vec<Discovered>,
    ) -> Shell {
        let catalog = match Catalog::load(&files.join(CATALOG_FILE), own_package) {
            Some(catalog) => catalog,
            None => {
                let catalog = Catalog::from_discovered(&discover(), own_package);
                save_log(catalog.save(&files.join(CATALOG_FILE)), CATALOG_FILE);
                catalog
            }
        };

        let usage_path = files.join(USAGE_FILE);
        let mut usage = UsageHistory::load(&usage_path);
        if !usage_path.exists() {
            // The apps that were already installed are not "new".
            usage.mark_baseline(catalog.apps().iter().map(|a| a.package.as_str()));
            save_log(usage.save(&usage_path), USAGE_FILE);
        }

        let config_path = files.join(CONFIG_FILE);
        let mut config = HomeConfig::load(&config_path).unwrap_or_else(|| {
            let config = HomeConfig::first_use(&catalog);
            save_log(config.save(&config_path), CONFIG_FILE);
            config
        });
        config.remove_uninstalled(&catalog);

        let preferred: Vec<&str> = locales.iter().map(String::as_str).collect();
        let mut app = LumeApp::new(catalog, usage, config, Translator::new(&preferred), clock);
        app.keys = crate::keys::KeyMap::load_or_heal(&files.join(KEYS_FILE));
        Shell {
            app,
            files: files.to_path_buf(),
            own_package: own_package.to_string(),
            text: None,
            display: None,
            list: DisplayList::new(),
            dirty: true,
        }
    }

    pub fn app(&self) -> &LumeApp {
        &self.app
    }

    /// The launcher, for tests of the shell.
    #[doc(hidden)]
    pub fn app_mut_for_test(&mut self) -> &mut LumeApp {
        &mut self.app
    }

    pub fn is_active(&self) -> bool {
        self.text.is_some()
    }

    /// The window is on screen with this size in pixels: creates what drawing needs.
    pub fn wake(&mut self, width: u32, height: u32) {
        let display = Display::new(width, height);
        match &mut self.text {
            Some(text) => text.set_scale(display.scale),
            None => {
                // Without Android's fonts.xml (desktop) there is no system fallback.
                self.text = Some(match SystemFonts::android() {
                    Ok(fonts) => TextEngine::with_fallback(display.scale, Box::new(fonts)),
                    Err(_) => TextEngine::new(display.scale),
                });
                self.app.came_to_foreground();
            }
        }
        // The pictures are kept at the size they reach on screen at this scale: a new scale drops
        // them, and the platform makes the texture again at the new size (`icon_layout`).
        self.app
            .icons
            .set_layout(crate::app_icons::AtlasLayout::for_scale(display.scale));
        self.display = Some(display);
        self.dirty = true;
    }

    /// The sizes of the icon texture and of its cells, at the current scale.
    pub fn icon_layout(&self) -> crate::app_icons::AtlasLayout {
        self.app.icons.layout()
    }

    /// Another app is in front: drops the glyph atlas and every other buffer used for drawing.
    pub fn sleep(&mut self) {
        self.app.went_to_background();
        self.app.icons.clear();
        if let Some(text) = &mut self.text {
            text.release();
        }
        self.text = None;
        self.display = None;
        self.list = DisplayList::new();
    }

    pub fn viewport(&self) -> Option<Viewport> {
        self.display.map(|d| d.viewport())
    }

    /// Up to `limit` apps (package and activity) whose icon is not in the icon texture yet and that
    /// the platform has not already failed to load: the visible tiles the splash waits for first,
    /// then the favorites, then the app list.
    pub fn icons_wanted(
        &self,
        skip: &std::collections::HashSet<String>,
        limit: usize,
    ) -> Vec<(String, String)> {
        if limit == 0 || !self.is_active() || !self.app.icons.has_room() {
            return Vec::new();
        }
        let splash = self
            .app
            .splash_missing_tiles()
            .iter()
            .filter_map(|p| self.app.catalog.get(p));
        let favorites = self
            .app
            .config
            .favorites
            .iter()
            .filter_map(|p| self.app.catalog.get(p));
        let mut seen = std::collections::HashSet::new();
        splash
            .chain(favorites)
            .chain(self.app.catalog.apps().iter())
            .filter(|app| {
                !self.app.icons.has(&app.package)
                    && !skip.contains(&app.package)
                    && seen.insert(app.package.as_str())
            })
            .take(limit)
            .map(|app| (app.package.clone(), app.activity.clone()))
            .collect()
    }

    /// Up to `limit` favorites (package and activity) that have no dock icon in the icon texture
    /// yet and that the platform has not already failed to load, in dock order. Dock cells of apps
    /// that stopped being favorites are given back first. Only the favorites are ever asked for.
    pub fn dock_icons_wanted(
        &mut self,
        skip: &std::collections::HashSet<String>,
        limit: usize,
    ) -> Vec<(String, String)> {
        if !self.is_active() {
            return Vec::new();
        }
        let favorites = self.app.config.favorites.clone();
        self.app.icons.release_dock_except(&favorites);
        favorites
            .iter()
            .filter_map(|p| self.app.catalog.get(p))
            .filter(|app| !self.app.icons.has_dock(&app.package) && !skip.contains(&app.package))
            .map(|app| (app.package.clone(), app.activity.clone()))
            .take(limit)
            .collect()
    }

    /// Reserves the dock cell of `package`; the platform uploads its icon to the returned region
    /// (`[x, y, width, height]`).
    /// `now_ms` is when it arrives: with the interface on screen, it fades in from then.
    pub fn reserve_dock_icon(&mut self, package: &str, now_ms: u64) -> Option<[u32; 4]> {
        let region = self.app.icons.reserve_dock(package)?;
        self.app.icon_arrived(package, true, now_ms);
        self.dirty = true;
        Some(region)
    }

    /// Reserves the texture cell of `package` for its banner or icon; the platform uploads the pixels to the
    /// returned region (`[x, y, width, height]`).
    /// `now_ms` is when it arrives: with the interface on screen, it fades in from then.
    pub fn reserve_icon(
        &mut self,
        package: &str,
        art: crate::app_icons::Art,
        now_ms: u64,
    ) -> Option<[u32; 4]> {
        let region = self.app.icons.reserve(package, art)?;
        self.app.icon_arrived(package, false, now_ms);
        self.dirty = true;
        Some(region)
    }

    /// The atlas the next frame needs uploaded.
    pub fn text_engine(&mut self) -> Option<&mut TextEngine> {
        self.text.as_mut()
    }

    /// Passes a key to the launcher, saves what changed, and returns what the platform must do.
    pub fn key(&mut self, key: Key, now_ms: u64) -> Vec<Effect> {
        let effects = self.app.handle_key(key, now_ms);
        self.dirty = true;
        self.finish(effects)
    }

    /// Passes a raw hardware keycode to the launcher, saves what changed, and returns what the platform must do.
    pub fn raw_key(&mut self, code: i32, now_ms: u64) -> Vec<Effect> {
        let effects = self.app.handle_keycode(code, now_ms);
        self.dirty = true;
        self.finish(effects)
    }

    /// Saves data the launcher asked to save and hands over the effects for the platform.
    fn finish(&mut self, effects: Vec<Effect>) -> Vec<Effect> {
        let mut platform = Vec::new();
        for effect in effects {
            match effect {
                Effect::SaveConfig => save_log(
                    self.app.config.save(&self.files.join(CONFIG_FILE)),
                    CONFIG_FILE,
                ),
                Effect::SaveUsage => save_log(
                    self.app.usage.save(&self.files.join(USAGE_FILE)),
                    USAGE_FILE,
                ),
                Effect::SaveKeys => {
                    save_log(self.app.keys.save(&self.files.join(KEYS_FILE)), KEYS_FILE)
                }
                other => platform.push(other),
            }
        }
        platform
    }

    /// The platform could not give the dock icon (`dock`) or the tile image of `package`: the
    /// splash stops waiting for it.
    pub fn icon_failed(&mut self, package: &str, dock: bool) {
        self.app.icon_failed(package, dock);
        if self.app.in_splash() {
            self.dirty = true;
        }
    }

    /// While the splash waits for icons, how long until it stops waiting (ms).
    pub fn splash_time_left(&self, now_ms: u64) -> Option<u64> {
        self.app
            .splash_deadline()
            .map(|at| at.saturating_sub(now_ms))
    }

    /// Whether the interface is being revealed after the splash.
    pub fn is_revealing(&self) -> bool {
        self.app.is_revealing()
    }

    /// Whether the interface became ready (the splash ended) since the last call.
    pub fn take_interface_ready(&mut self) -> bool {
        self.app.take_interface_ready()
    }

    /// The platform could not open `package` (it was removed meanwhile).
    pub fn app_not_found(&mut self, now_ms: u64) {
        self.app.app_not_found(now_ms);
        self.dirty = true;
    }

    /// Whether the system has a TV inputs panel, which decides if the top bar has its button.
    pub fn set_inputs_available(&mut self, available: bool) {
        if available != self.app.inputs_available {
            self.app.inputs_available = available;
            self.dirty = true;
        }
    }

    /// Wall time, time zone and 12/24-hour setting.
    pub fn set_clock(&mut self, clock: WallClock) {
        if clock != self.app.clock {
            self.app.clock = clock;
            self.dirty = true;
        }
    }

    /// Only the seconds since the epoch moved on (the zone and format are the same).
    pub fn tick_clock(&mut self, unix_secs: u64) {
        let clock = WallClock {
            unix_secs,
            ..self.app.clock
        };
        // The screen only changes when the minute does.
        if clock.text() != self.app.clock.text() {
            self.dirty = true;
        }
        self.app.clock = clock;
    }

    /// The language list of the system changed.
    pub fn set_locales(&mut self, locales: &[String]) {
        let preferred: Vec<&str> = locales.iter().map(String::as_str).collect();
        let translator = Translator::new(&preferred);
        if translator.language() != self.app.translator.language() {
            self.app.translator = translator;
            // Glyphs of the old language may not be needed any more.
            if let Some(text) = &mut self.text {
                text.clear();
            }
            self.dirty = true;
        }
    }

    /// Apps were installed, updated or removed. `now_secs` is the wall time.
    pub fn apply_changes(&mut self, changed: &[Discovered], removed: &[String], now_secs: u64) {
        let changes = self.app.catalog.apply(changed, removed);
        self.after_catalog_changes(&changes, now_secs);
    }

    /// The catalog was scanned from scratch.
    pub fn rescan(&mut self, discovered: &[Discovered], now_secs: u64) {
        let changes = self.app.catalog.replace_with(discovered);
        self.after_catalog_changes(&changes, now_secs);
    }

    fn after_catalog_changes(&mut self, changes: &crate::catalog::Changes, now_secs: u64) {
        if changes.is_empty() {
            return;
        }
        for package in &changes.added {
            self.app.usage.mark_seen(package, now_secs);
        }
        for package in &changes.removed {
            self.app.usage.forget(package);
        }
        if self.app.config.remove_uninstalled(&self.app.catalog) {
            save_log(
                self.app.config.save(&self.files.join(CONFIG_FILE)),
                CONFIG_FILE,
            );
        }
        self.app.catalog_changed();
        save_log(
            self.app.catalog.save(&self.files.join(CATALOG_FILE)),
            CATALOG_FILE,
        );
        save_log(
            self.app.usage.save(&self.files.join(USAGE_FILE)),
            USAGE_FILE,
        );
        self.dirty = true;
    }

    pub fn own_package(&self) -> &str {
        &self.own_package
    }

    /// How long the event loop may sleep: `Some(Duration::ZERO)` when a frame is due now,
    /// `None` when nothing will change without an event.
    pub fn next_timeout(&self, now_ms: u64) -> Option<Duration> {
        if !self.is_active() {
            return None;
        }
        if self.dirty {
            return Some(Duration::ZERO);
        }
        self.app
            .next_deadline(now_ms)
            .map(|at| Duration::from_millis(at.saturating_sub(now_ms)))
    }

    /// How long until the TV statistics are due, when they are being read (the settings are on
    /// screen). Nothing is read during an animation, so the frames stay on time.
    pub fn stats_timeout(&self, now_ms: u64) -> Option<Duration> {
        if !self.is_active() || self.is_busy(now_ms) {
            return None;
        }
        self.app
            .stats_deadline(now_ms)
            .map(|at| Duration::from_millis(at.saturating_sub(now_ms)))
    }

    /// Whether a statistics reading is due now.
    pub fn stats_due(&self, now_ms: u64) -> bool {
        self.stats_timeout(now_ms) == Some(Duration::ZERO)
    }

    /// The platform could not read the statistics: the next try comes after the usual interval.
    pub fn stats_failed(&mut self, now_ms: u64) {
        self.app.stats_retry(now_ms);
    }

    /// A statistics reading from the platform.
    pub fn set_stats(&mut self, stats: crate::stats::Stats, now_ms: u64) {
        self.app.apply_stats(stats, now_ms);
        self.dirty = true;
    }

    /// Whether the TV information screen waits for the platform's reading and the interface is at
    /// rest (the reading takes a few milliseconds, so it never lands in the middle of a move).
    pub fn device_info_due(&self, now_ms: u64) -> bool {
        self.is_active() && !self.is_busy(now_ms) && self.app.wants_device_info()
    }

    /// The platform's reading of the TV for the information screen.
    pub fn set_device_info(&mut self, info: crate::device_info::DeviceInfo) {
        self.app.apply_device_info(info);
        self.dirty = true;
    }

    /// Whether the screen is changing right now (a key was just handled, or something moves; the
    /// fades of the splash logo and of arriving icons do not count). Slow work that can wait, like
    /// loading icons, should wait for this to end.
    pub fn is_busy(&self, now_ms: u64) -> bool {
        self.is_active() && (self.dirty || self.app.is_moving(now_ms))
    }

    /// Whether the screen must be redrawn at `now_ms`.
    /// Asks for a frame although nothing in the interface changed (a wallpaper prepared in the
    /// background is ready to be swapped in).
    pub fn invalidate(&mut self) {
        self.dirty = true;
    }

    pub fn frame_due(&self, now_ms: u64) -> bool {
        self.is_active() && self.next_timeout(now_ms) == Some(Duration::ZERO)
    }

    /// Builds the frame to draw, or `None` while dormant.
    pub fn build_frame(&mut self, now_ms: u64) -> Option<Frame> {
        let display = self.display?;
        let text = self.text.as_mut()?;
        for _ in 0..MAX_FRAME_PASSES {
            self.list.clear();
            text.begin_frame();
            let mut painter = Painter {
                list: &mut self.list,
                text,
                display,
            };
            self.app.draw(&mut painter, now_ms);
            if !text.end_frame() {
                break;
            }
        }
        self.dirty = false;
        Some(Frame::build(&self.list))
    }

    /// The size of the buffer to render for a screen of `width` x `height` pixels.
    pub fn render_size(width: u32, height: u32) -> (u32, u32) {
        display::render_size(width, height)
    }
}

fn save_log(result: std::io::Result<()>, name: &str) {
    if let Err(e) = result {
        log::warn!("cannot save {name}: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::ids;
    use crate::app_icons::Art;
    use crate::home::config::HomeConfig;

    const OWN: &str = "app.lume.tv";
    const NETFLIX: &str = "com.netflix.ninja";
    const YOUTUBE: &str = "com.google.android.youtube.tv";
    const NOW_SECS: u64 = 1_792_446_300;

    fn entry(package: &str, label: &str) -> Discovered {
        Discovered {
            package: package.to_string(),
            label: label.to_string(),
            activity: format!("{package}.Main"),
            leanback: true,
            system: false,
            icon: None,
        }
    }

    fn device() -> Vec<Discovered> {
        vec![
            entry(NETFLIX, "Netflix"),
            entry(YOUTUBE, "YouTube"),
            entry(OWN, "lume"),
            entry("com.demo.x", "Demo"),
        ]
    }

    fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lume-shell-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn clock() -> WallClock {
        WallClock {
            unix_secs: NOW_SECS,
            utc_offset_secs: 0,
            is_24h: true,
        }
    }

    fn start(dir: &Path) -> Shell {
        Shell::start(dir, OWN, &["pt-BR".to_string()], clock(), &mut device)
    }

    fn packages(wanted: Vec<(String, String)>) -> Vec<String> {
        wanted.into_iter().map(|(package, _)| package).collect()
    }

    /// Draws the frames the loop would ask for until nothing moves; returns the time it settled.
    /// Draws frames until nothing moves: past the splash (which waits up to 600 ms for icons no
    /// one loads here) and its reveal.
    fn settle(shell: &mut Shell, mut now: u64) -> u64 {
        for _ in 0..300 {
            if shell.frame_due(now) {
                shell.build_frame(now).unwrap();
            } else if !shell.app().in_splash() {
                return now;
            }
            now += 16;
        }
        panic!("the animation never settled");
    }

    fn texts(list: &DisplayList) -> usize {
        list.cmds().len()
    }

    #[test]
    fn the_first_run_scans_the_device_and_creates_every_file() {
        let dir = dir("first");
        let mut scans = 0;
        let shell = Shell::start(&dir, OWN, &[], clock(), &mut || {
            scans += 1;
            device()
        });
        assert_eq!(scans, 1);
        assert_eq!(shell.app().catalog.len(), 3, "lume itself is not listed");
        assert!(!shell.app().catalog.contains(OWN));
        for file in [CONFIG_FILE, CATALOG_FILE, USAGE_FILE, KEYS_FILE] {
            assert!(dir.join(file).exists(), "{file} was not created");
        }
        assert_eq!(shell.app().config.favorites, [NETFLIX, YOUTUBE]);
        // Apps that were already there are not "new".
        assert!(!shell.app().usage.is_new(NETFLIX, NOW_SECS));
    }

    #[test]
    fn later_runs_use_the_cache_without_scanning() {
        let dir = dir("cache");
        start(&dir);
        let mut scans = 0;
        let shell = Shell::start(&dir, OWN, &[], clock(), &mut || {
            scans += 1;
            vec![]
        });
        assert_eq!(scans, 0, "the saved catalog is enough");
        assert_eq!(shell.app().catalog.len(), 3);
    }

    #[test]
    fn a_corrupt_cache_triggers_a_new_scan() {
        let dir = dir("corrupt");
        start(&dir);
        std::fs::write(dir.join(CATALOG_FILE), "garbage").unwrap();
        let mut scans = 0;
        let shell = Shell::start(&dir, OWN, &[], clock(), &mut || {
            scans += 1;
            device()
        });
        assert_eq!(scans, 1);
        assert_eq!(shell.app().catalog.len(), 3);
    }

    #[test]
    fn the_saved_configuration_survives_a_restart() {
        let dir = dir("restart");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        shell.build_frame(0).unwrap();
        // Open the editor and move the first favorite right: the Apps tab, the Lume tile, its
        // settings, "Tela Início".
        for key in [Key::Up, Key::Right, Key::Ok, Key::Down] {
            shell.key(key, 0);
            shell.build_frame(0).unwrap();
        }
        assert!(shell.app_mut_for_test().focus_for_test(2999));
        for key in [Key::Ok, Key::Ok] {
            shell.key(key, 0);
            shell.build_frame(0).unwrap();
        }
        let right = 4400 + 1;
        assert!(shell.app_mut_for_test().focus_for_test(right));
        let effects = shell.key(Key::Ok, 100);
        assert!(
            effects.is_empty(),
            "saving is the shell's job, not the platform's"
        );
        assert_eq!(shell.app().config.favorites, [YOUTUBE, NETFLIX]);

        let again = start(&dir);
        assert_eq!(
            again.app().config.favorites,
            [YOUTUBE, NETFLIX],
            "the order persisted"
        );
    }

    #[test]
    fn opening_an_app_reaches_the_platform_and_is_recorded() {
        let dir = dir("open");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        shell.build_frame(0).unwrap();
        // The first favorite has the focus at the start.
        let effects = shell.key(Key::Ok, 10);
        assert_eq!(
            effects,
            [Effect::OpenApp {
                package: NETFLIX.into(),
                activity: format!("{NETFLIX}.Main")
            }]
        );
        let history = UsageHistory::load(&dir.join(USAGE_FILE));
        assert_eq!(history.last_opened(NETFLIX), Some(NOW_SECS));
    }

    #[test]
    fn going_to_sleep_frees_the_drawing_memory_and_waking_restores_it() {
        let dir = dir("sleep");
        let mut shell = start(&dir);
        assert!(!shell.is_active());
        shell.wake(1920, 1080);
        assert!(shell.is_active());
        let frame = shell.build_frame(0).unwrap();
        assert!(!frame.batches.is_empty());
        assert!(shell.text_engine().unwrap().cached_glyphs() > 0);

        shell.sleep();
        assert!(!shell.is_active());
        assert!(shell.text_engine().is_none(), "the glyph atlas is gone");
        assert!(
            shell.build_frame(10).is_none(),
            "nothing is drawn while dormant"
        );
        assert!(shell.viewport().is_none());
        assert_eq!(shell.next_timeout(10), None, "no wake-ups while dormant");

        // The user's data is untouched.
        assert_eq!(shell.app().config.favorites, [NETFLIX, YOUTUBE]);
        shell.wake(1920, 1080);
        assert!(shell.build_frame(20).is_some());
        assert_eq!(
            shell.app().focused(),
            Some(ids::FAVORITE_BASE),
            "the screen is where it was"
        );
    }

    #[test]
    fn frames_are_built_only_when_something_changed() {
        let dir = dir("frames");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        assert!(shell.frame_due(0), "the first frame");
        shell.build_frame(0).unwrap();
        // Once the first animation is over, the next change is the clock's minute.
        let now = settle(&mut shell, 16);
        let wait = shell.next_timeout(now).unwrap();
        assert!(wait > Duration::ZERO, "idle: {wait:?}");
        assert!(!shell.frame_due(now));
        shell.key(Key::Down, now);
        assert!(shell.frame_due(now), "a key press needs a frame");
    }

    #[test]
    fn the_clock_only_asks_for_a_frame_when_the_minute_changes() {
        let dir = dir("clock");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        shell.build_frame(0).unwrap();
        settle(&mut shell, 16);
        shell.tick_clock(NOW_SECS + 10);
        assert!(!shell.frame_due(10_000), "the same minute");
        shell.tick_clock(NOW_SECS + 61);
        assert!(shell.frame_due(61_000), "a new minute");
        shell.build_frame(61_000).unwrap();
        shell.set_clock(WallClock {
            is_24h: false,
            ..shell.app().clock
        });
        assert!(shell.frame_due(62_000), "the 12/24-hour setting changed");
    }

    #[test]
    fn a_language_change_redraws_in_the_new_language() {
        let dir = dir("locale");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        assert_eq!(
            shell.app().translator.get(crate::i18n::StringId::TabHome),
            "Início"
        );
        shell.build_frame(0).unwrap();
        shell.set_locales(&["en-US".to_string()]);
        assert!(shell.frame_due(1));
        assert_eq!(
            shell.app().translator.get(crate::i18n::StringId::TabHome),
            "Home"
        );
        // The same language again changes nothing.
        shell.build_frame(1).unwrap();
        shell.set_locales(&["en-GB".to_string()]);
        assert!(!shell.frame_due(2));
        // The configuration is kept.
        assert_eq!(shell.app().config.favorites, [NETFLIX, YOUTUBE]);
    }

    #[test]
    fn installing_an_app_updates_the_catalog_the_history_and_the_files() {
        let dir = dir("install");
        let mut shell = start(&dir);
        shell.apply_changes(
            &[entry("com.globo.globotv", "Globoplay")],
            &[],
            NOW_SECS + 5,
        );
        assert!(shell.app().catalog.contains("com.globo.globotv"));
        assert!(shell.app().usage.is_new("com.globo.globotv", NOW_SECS + 6));
        assert_eq!(
            Catalog::load(&dir.join(CATALOG_FILE), OWN).unwrap().len(),
            4
        );
        // The change is reported once: a repeat does not touch the history.
        shell.apply_changes(
            &[entry("com.globo.globotv", "Globoplay")],
            &[],
            NOW_SECS + 500,
        );
        assert_eq!(
            shell.app().usage.first_seen("com.globo.globotv"),
            Some(NOW_SECS + 5)
        );
    }

    #[test]
    fn uninstalling_removes_the_app_from_the_favorites() {
        let dir = dir("uninstall");
        let mut shell = start(&dir);
        shell.apply_changes(&[], &[NETFLIX.to_string()], NOW_SECS);
        assert_eq!(shell.app().config.favorites, [YOUTUBE]);
        assert_eq!(
            HomeConfig::load(&dir.join(CONFIG_FILE)).unwrap().favorites,
            [YOUTUBE],
            "saved"
        );
        assert_eq!(
            shell.app().usage.first_seen(NETFLIX),
            None,
            "the history of a removed app is forgotten"
        );
    }

    #[test]
    fn a_full_rescan_applies_the_difference() {
        let dir = dir("rescan");
        let mut shell = start(&dir);
        shell.rescan(
            &[entry(YOUTUBE, "YouTube"), entry("com.new", "New")],
            NOW_SECS + 9,
        );
        assert!(!shell.app().catalog.contains(NETFLIX));
        assert!(shell.app().catalog.contains("com.new"));
        assert_eq!(shell.app().config.favorites, [YOUTUBE]);
    }

    #[test]
    fn a_vanished_app_shows_a_message() {
        let dir = dir("missing");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        shell.build_frame(0).unwrap();
        shell.app_not_found(5);
        assert!(shell.frame_due(5));
        let frame = shell.build_frame(5).unwrap();
        assert!(texts(&shell.list) > 0 && !frame.batches.is_empty());
        assert_eq!(shell.app().toast_text(), Some("App não encontrado"));
    }

    #[test]
    fn changing_the_screen_size_brings_the_icons_back_at_the_new_size() {
        let dir = dir("resize-icons");
        let mut shell = start(&dir);
        let none = std::collections::HashSet::new();
        shell.wake(1280, 720);
        let small = shell.icon_layout();
        assert_eq!(shell.reserve_dock_icon(NETFLIX, 0).unwrap()[2], 104);
        assert_eq!(
            shell.reserve_icon(NETFLIX, Art::Banner, 0).unwrap()[2..],
            [212, 119]
        );
        // Woken again at the same size, nothing is fetched again.
        shell.wake(1280, 720);
        assert_eq!(shell.icon_layout(), small);
        assert!(shell.app().icons.has_dock(NETFLIX));

        shell.wake(1920, 1080);
        let large = shell.icon_layout();
        assert_eq!((large.width, large.dock_cell), (1280, 156));
        // The pictures of the old size are gone and asked for again, in cells of the new one.
        assert!(!shell.app().icons.has(NETFLIX) && !shell.app().icons.has_dock(NETFLIX));
        assert!(packages(shell.dock_icons_wanted(&none, 5)).contains(&NETFLIX.to_string()));
        assert!(packages(shell.icons_wanted(&none, 5)).contains(&NETFLIX.to_string()));
        assert_eq!(shell.reserve_dock_icon(NETFLIX, 0).unwrap()[2], 156);
        assert_eq!(
            shell.reserve_icon(NETFLIX, Art::Banner, 0).unwrap()[2..],
            [320, 180]
        );
    }

    #[test]
    fn changing_the_screen_size_rebuilds_the_text_for_the_new_scale() {
        let dir = dir("resize");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        shell.build_frame(0).unwrap();
        assert_eq!(shell.text_engine().unwrap().scale(), 1.0);
        shell.wake(1920, 1080);
        assert_eq!(shell.text_engine().unwrap().scale(), 1.5);
        assert_eq!(
            shell.text_engine().unwrap().cached_glyphs(),
            0,
            "glyphs of the old scale are gone"
        );
        assert_eq!(shell.viewport().unwrap().scale, 1.5);
    }

    #[test]
    fn the_render_size_is_capped() {
        assert_eq!(Shell::render_size(3840, 2160), (1920, 1080));
        assert_eq!(Shell::render_size(1280, 720), (1280, 720));
    }

    #[test]
    fn dock_icons_are_wanted_only_for_favorites_one_batch_at_a_time_and_only_while_active() {
        let dir = dir("dock-icons");
        let mut shell = start(&dir);
        let none = std::collections::HashSet::new();
        assert!(shell.dock_icons_wanted(&none, 5).is_empty(), "dormant");
        shell.wake(1280, 720);
        let favorites = shell.app().config.favorites.clone();
        assert_eq!(favorites, [NETFLIX, YOUTUBE]);
        // Only the favorites, a few at a time, in dock order.
        let first = shell.dock_icons_wanted(&none, 1);
        assert_eq!(first, [(NETFLIX.to_string(), format!("{NETFLIX}.Main"))]);
        let region = shell.reserve_dock_icon(NETFLIX, 0).unwrap();
        assert_eq!(region[2..], [shell.icon_layout().dock_cell; 2]);
        let next = shell.dock_icons_wanted(&none, 5);
        assert_eq!(
            next.iter().map(|(p, _)| p.as_str()).collect::<Vec<_>>(),
            [YOUTUBE]
        );
        // The ones the platform failed on are not asked for again.
        let failed = std::collections::HashSet::from([YOUTUBE.to_string()]);
        assert!(shell.dock_icons_wanted(&failed, 5).is_empty());
    }

    #[test]
    fn a_dock_cell_goes_back_when_the_app_stops_being_a_favorite() {
        let dir = dir("dock-release");
        let mut shell = start(&dir);
        let none = std::collections::HashSet::new();
        shell.wake(1280, 720);
        shell.reserve_dock_icon(NETFLIX, 0).unwrap();
        shell.reserve_dock_icon(YOUTUBE, 0).unwrap();
        assert!(shell.dock_icons_wanted(&none, 5).is_empty(), "all loaded");
        shell.app_mut_for_test().config.remove_favorite(0);
        assert!(shell.dock_icons_wanted(&none, 5).is_empty());
        assert!(
            !shell.app().icons.has_dock(NETFLIX),
            "its cell was given back"
        );
        assert!(shell.app().icons.has_dock(YOUTUBE));
    }

    #[test]
    fn the_dock_icons_go_away_when_lume_sleeps() {
        let dir = dir("dock-sleep");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        shell.reserve_dock_icon(NETFLIX, 0).unwrap();
        assert!(shell.app().icons.has_dock(NETFLIX));
        shell.sleep();
        assert!(!shell.app().icons.has_dock(NETFLIX));
        shell.wake(1280, 720);
        let none = std::collections::HashSet::new();
        assert_eq!(
            shell.dock_icons_wanted(&none, 5).len(),
            2,
            "loaded again at the next wake"
        );
    }

    #[test]
    fn icons_are_wanted_favorites_first_and_only_while_active() {
        let dir = dir("icons");
        let mut shell = start(&dir);
        let none = std::collections::HashSet::new();
        assert!(shell.icons_wanted(&none, 5).is_empty(), "dormant");

        shell.wake(1280, 720);
        let wanted = packages(shell.icons_wanted(&none, 5));
        assert_eq!(wanted, [NETFLIX, YOUTUBE, "com.demo.x"]);
        assert_eq!(shell.icons_wanted(&none, 1).len(), 1, "the limit holds");

        let region = shell.reserve_icon(NETFLIX, Art::Icon, 0).unwrap();
        assert_eq!(region[2], shell.icon_layout().cell[0]);
        let wanted = packages(shell.icons_wanted(&none, 5));
        assert_eq!(wanted, [YOUTUBE, "com.demo.x"]);

        let failed: std::collections::HashSet<String> = [YOUTUBE.to_string()].into();
        assert_eq!(packages(shell.icons_wanted(&failed, 5)), ["com.demo.x"]);
    }

    #[test]
    fn the_icons_go_away_when_lume_sleeps() {
        let dir = dir("icons-sleep");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        shell.reserve_icon(NETFLIX, Art::Icon, 0).unwrap();
        shell.sleep();
        shell.wake(1280, 720);
        assert!(!shell.app().icons.has(NETFLIX));
        assert_eq!(shell.icons_wanted(&Default::default(), 1).len(), 1);
    }

    #[test]
    fn frames_keep_coming_while_an_animation_runs() {
        let dir = dir("animation");
        let mut shell = start(&dir);
        shell.wake(1280, 720);
        shell.build_frame(0);
        shell.key(Key::Right, 1_000);
        shell.build_frame(1_000);
        // The focus animation takes a few hundred milliseconds: the loop sleeps one frame at a
        // time and must draw each time it wakes up.
        let mut now = 1_000;
        let mut drawn = 0;
        while now < 1_300 {
            let wait = shell.next_timeout(now).expect("an animation needs frames");
            now += wait.as_millis() as u64;
            if shell.frame_due(now) {
                shell.build_frame(now);
                drawn += 1;
            } else if wait.is_zero() {
                break;
            }
        }
        assert!(
            drawn >= 5,
            "only {drawn} frames of the animation were drawn"
        );
    }

    #[test]
    fn shell_start_creates_keys_file_and_recreates_on_clean_data() {
        let dir = dir("keys_first_run");
        let keys_path = dir.join(KEYS_FILE);
        assert!(!keys_path.exists());

        let shell = start(&dir);
        assert!(keys_path.exists());
        assert!(shell.app().keys.is_all_default());

        // Simulate "Clear data" in Android settings by deleting keys.v1
        std::fs::remove_file(&keys_path).unwrap();
        assert!(!keys_path.exists());

        let shell2 = start(&dir);
        assert!(keys_path.exists());
        assert!(shell2.app().keys.is_all_default());
    }

    #[test]
    fn shell_start_respects_custom_keys_and_heals_corrupt_file() {
        let dir = dir("keys_custom");
        let keys_path = dir.join(KEYS_FILE);
        let custom = "lume=1\nup=19\ndown=20\nleft=21\nright=22\nok=23,66,160,96\nback=4,111,97\nhome=3\nmenu=165\n#end\n";
        std::fs::write(&keys_path, custom).unwrap();

        let shell = start(&dir);
        assert_eq!(shell.app().keys.codes(Key::Menu), &[165]);
        assert_eq!(shell.app().keys.from_android(165), Some(Key::Menu));

        // Corrupt keys.v1
        std::fs::write(&keys_path, "lume=1\nmenu=165\n").unwrap(); // missing #end
        let shell2 = start(&dir);
        assert!(shell2.app().keys.is_all_default());
        let healed = std::fs::read_to_string(&keys_path).unwrap();
        assert!(healed.ends_with("#end\n"));
    }

    #[test]
    fn effect_save_keys_persists_changes() {
        let dir = dir("keys_save_effect");
        let mut shell = start(&dir);
        assert!(shell.app().keys.is_all_default());

        // Modify keys
        shell.app_mut_for_test().keys.set(Key::Menu, vec![165]);
        assert_eq!(shell.app().keys.codes(Key::Menu), &[165]);

        // Process SaveKeys effect through shell.finish
        let remaining = shell.finish(vec![Effect::SaveKeys]);
        assert!(remaining.is_empty(), "SaveKeys handled by shell");

        // Verify keys.v1 has updated mapping
        let reloaded = crate::keys::KeyMap::load_or_heal(&dir.join(KEYS_FILE));
        assert_eq!(reloaded.codes(Key::Menu), &[165]);
        assert_eq!(reloaded.from_android(165), Some(Key::Menu));
    }
}
