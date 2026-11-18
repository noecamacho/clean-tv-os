//! HOME on LG webOS: the event loop.
//!
//! A trimmed port of the Android loop (`lume-app/src/android/mod.rs`): SDL gives the window,
//! the GLES2 context and the remote's keys; the Luna bus replaces the JNI bridge. The window
//! lives as long as the process, so there is no park/resume; frames are drawn in full (no
//! buffer-age partial redraw yet — measure first on this GPU).

#[cfg(target_arch = "arm")]
mod bridge;
mod icons;
#[cfg(target_arch = "arm")]
mod luna;
#[cfg(target_arch = "arm")]
mod sdl;
#[cfg(target_arch = "arm")]
mod webos_compat;

#[cfg(not(target_arch = "arm"))]
fn main() {
    eprintln!("home runs only on the TV (armv7 webOS); this build keeps the tests compilable");
}

#[cfg(target_arch = "arm")]
fn main() {
    tv::main()
}

#[cfg(target_arch = "arm")]
mod tv {
    use crate::bridge::Bridge;
    use crate::sdl;
    use lume_core::app::{Effect, Key};
    use lume_core::render::Gles2Backend;
    use lume_core::shell::Shell;
    use lume_core::theme::vars;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::OnceLock;
    use std::time::{Duration, Instant};

    /// Where the launcher persists its files; survives package reinstalls (see Luma's storage).
    const DATA_DIR: &str = "/media/developer/app.home.tv";
    /// How long OK is held before it opens the menu instead of acting.
    const LONG_PRESS: Duration = Duration::from_millis(500);
    /// How often the open Control Center reads the TV's volume and OLED light.
    const LEVELS_EVERY: Duration = Duration::from_millis(300);
    /// The loop never sleeps longer than this: a wallpaper decoded in the background has no
    /// SDL waker here, and one wake-up a second costs nothing on mains power.
    const MAX_WAIT: Duration = Duration::from_millis(1000);
    /// Icons fetched per call, and the turn budgets, as on Android.
    const ICONS_PER_TURN: usize = 8;
    const SPLASH_BATCH: Duration = Duration::from_millis(40);
    const IDLE_BATCH: Duration = Duration::from_millis(16);
    const FADE_BATCH: Duration = Duration::from_millis(6);
    const SPLASH_MARGIN_MS: u64 = 60;

    /// Set by the wallpaper's worker thread when a decoded wallpaper is ready.
    static WALLPAPER_READY: AtomicBool = AtomicBool::new(false);
    /// Directories the wallpaper assets are read from (the package, then the data dir).
    static ASSET_DIRS: OnceLock<Vec<PathBuf>> = OnceLock::new();

    fn wallpaper_ready() {
        WALLPAPER_READY.store(true, Ordering::SeqCst);
    }

    fn read_wallpaper_asset(name: &str) -> Option<Vec<u8>> {
        ASSET_DIRS
            .get()?
            .iter()
            .find_map(|dir| std::fs::read(dir.join(name)).ok())
    }

    struct StderrLogger {
        started: Instant,
    }

    impl log::Log for StderrLogger {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= log::Level::Info
        }

        fn log(&self, record: &log::Record) {
            if self.enabled(record.metadata()) {
                eprintln!(
                    "home {:>7.1}s {} {}",
                    self.started.elapsed().as_secs_f32(),
                    record.level(),
                    record.args()
                );
            }
        }

        fn flush(&self) {}
    }

    fn map_key(scancode: i32) -> Option<Key> {
        Some(match scancode {
            sdl::SCANCODE_UP => Key::Up,
            sdl::SCANCODE_DOWN => Key::Down,
            sdl::SCANCODE_LEFT => Key::Left,
            sdl::SCANCODE_RIGHT => Key::Right,
            sdl::SCANCODE_RETURN | sdl::SCANCODE_KP_ENTER => Key::Ok,
            sdl::SCANCODE_WEBOS_BACK | sdl::SCANCODE_ESCAPE => Key::Back,
            sdl::SCANCODE_WEBOS_HOME => Key::Home,
            _ => return None,
        })
    }

    struct Runtime {
        window: sdl::Window,
        backend: Gles2Backend,
        bridge: Bridge,
        shell: Shell,
        files: PathBuf,
        size: (u32, u32),
        started: Instant,
        running: bool,
        visible: bool,
        /// When the TV goes to sleep by the Control Center's timer.
        sleep_at: Option<Instant>,
        /// When OK went down, and whether that press already opened the menu.
        ok_down: Option<Instant>,
        ok_long: bool,
        pressed: bool,
        /// Whether the Control Center was on screen at the last key.
        in_controls: bool,
        levels_read: Instant,
        icons_failed: HashSet<String>,
        dock_failed: HashSet<String>,
        icons_pending: bool,
        first_frame_from: Option<Instant>,
    }

    pub fn main() {
        let started = Instant::now();
        let logger = Box::leak(Box::new(StderrLogger { started }));
        let _ = log::set_logger(logger);
        log::set_max_level(log::LevelFilter::Info);
        std::panic::set_hook(Box::new(|info| log::error!("panic: {info}")));
        log::info!("HOME {} starting on webOS", lume_version::VERSION);

        let files = PathBuf::from(DATA_DIR);
        if let Err(e) = std::fs::create_dir_all(&files) {
            log::error!("cannot create {}: {e}", files.display());
            return;
        }
        let package_assets = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf))
            .map(|dir| dir.join("assets/wallpaper"));
        let mut dirs = vec![files.join("wallpaper")];
        if let Some(assets) = package_assets {
            dirs.insert(0, assets);
        }
        let _ = ASSET_DIRS.set(dirs);
        lume_core::wallpaper::set_asset_reader(read_wallpaper_asset);
        lume_core::wallpaper::set_ready_notifier(wallpaper_ready);
        lume_core::wallpaper::set_cache_dir(files.join("cache"));

        let mut bridge = Bridge::default();
        let locales = bridge.locales();
        let clock = bridge.clock();
        let mut shell = {
            let bridge = &mut bridge;
            Shell::start(&files, "app.home.tv", &locales, clock, &mut || {
                bridge.scan_all()
            })
        };
        shell.set_inputs_available(true);

        let window = match sdl::Window::open("HOME", 1920, 1080) {
            Ok(window) => window,
            Err(e) => {
                log::error!("no window: {e}");
                return;
            }
        };
        let size = window.drawable_size();
        let (width, height) = Shell::render_size(size.0, size.1);
        let mut backend = match Gles2Backend::new(window.glow()) {
            Ok(backend) => backend,
            Err(e) => {
                log::error!("no renderer: {e}");
                return;
            }
        };
        backend.set_render_size(width, height);
        backend.set_async_wallpaper(true);
        shell.wake(width, height);
        log::info!("window {width}x{height} (drawable {}x{})", size.0, size.1);

        let mut runtime = Runtime {
            window,
            backend,
            bridge,
            shell,
            files,
            size: (width, height),
            started,
            running: true,
            visible: true,
            sleep_at: None,
            ok_down: None,
            ok_long: false,
            pressed: false,
            in_controls: false,
            levels_read: Instant::now(),
            icons_failed: HashSet::new(),
            dock_failed: HashSet::new(),
            icons_pending: false,
            first_frame_from: Some(started),
        };
        runtime.refresh_system_settings();
        runtime.run();
        log::info!("HOME stopped");
    }

    impl Runtime {
        fn now_ms(&self) -> u64 {
            self.started.elapsed().as_millis() as u64
        }

        fn run(&mut self) {
            while self.running {
                let timeout = self.next_timeout();
                if let Some(event) = self.window.wait_event(Some(timeout)) {
                    self.on_event(event);
                    while let Some(event) = self.window.poll_event() {
                        self.on_event(event);
                    }
                }
                if !self.running {
                    break;
                }
                let now = self.now_ms();
                if self.in_controls && self.visible && self.levels_read.elapsed() >= LEVELS_EVERY {
                    self.refresh_levels();
                }
                if self.sleep_at.is_some_and(|at| Instant::now() >= at) {
                    self.sleep_at = None;
                    self.shell.set_sleep_timer(0);
                    if !self.bridge.screen_off() {
                        log::warn!("the sleep timer could not turn the screen off");
                    }
                }
                if !self.ok_long && self.ok_down.is_some_and(|t| t.elapsed() >= LONG_PRESS) {
                    self.ok_long = true;
                    self.send_key(Key::Menu);
                }
                let pressed = self.ok_down.is_some() && !self.ok_long;
                if pressed != self.pressed {
                    self.pressed = pressed;
                    let now = self.now_ms();
                    self.shell.set_pressed(pressed, now);
                }
                if WALLPAPER_READY.swap(false, Ordering::SeqCst) {
                    self.shell.invalidate();
                }
                self.shell.tick_clock(unix_seconds());
                if self.visible && self.shell.frame_due(self.now_ms()) {
                    self.draw();
                }
                if self.shell.stats_due(now) {
                    match self.bridge.stats(&self.files) {
                        Some(stats) => self.shell.set_stats(stats, now),
                        None => self.shell.stats_failed(now),
                    }
                }
                if self.shell.device_info_due(now) {
                    let info = self.device_info();
                    self.shell.set_device_info(info);
                }
                if self.visible && !self.shell.is_busy(self.now_ms()) {
                    self.prepare_wallpaper();
                    self.load_icons_batch();
                }
            }
        }

        /// How long this turn may wait for an event.
        fn next_timeout(&self) -> Duration {
            if !self.visible {
                return MAX_WAIT;
            }
            let now = self.now_ms();
            if self.icons_pending && !self.shell.is_busy(now) {
                return Duration::ZERO;
            }
            let mut timeout = [self.shell.next_timeout(now), self.shell.stats_timeout(now)]
                .into_iter()
                .flatten()
                .min()
                .unwrap_or(MAX_WAIT);
            if let Some(down) = self.ok_down.filter(|_| !self.ok_long) {
                timeout = timeout.min(LONG_PRESS.saturating_sub(down.elapsed()));
            }
            if let Some(at) = self.sleep_at {
                timeout = timeout.min(at.saturating_duration_since(Instant::now()));
            }
            if self.in_controls {
                timeout = timeout.min(LEVELS_EVERY);
            }
            timeout.min(MAX_WAIT)
        }

        fn on_event(&mut self, event: sdl::Event) {
            match event.kind() {
                sdl::QUIT => self.running = false,
                sdl::WINDOWEVENT => match event.window_event() {
                    sdl::WINDOWEVENT_CLOSE => self.running = false,
                    sdl::WINDOWEVENT_HIDDEN | sdl::WINDOWEVENT_FOCUS_LOST => self.set_hidden(),
                    sdl::WINDOWEVENT_SHOWN
                    | sdl::WINDOWEVENT_EXPOSED
                    | sdl::WINDOWEVENT_RESTORED
                    | sdl::WINDOWEVENT_FOCUS_GAINED => self.set_visible(),
                    _ => {}
                },
                sdl::KEYDOWN => {
                    let scancode = event.key_scancode();
                    if scancode == sdl::SCANCODE_WEBOS_EXIT {
                        self.running = false;
                        return;
                    }
                    match map_key(scancode) {
                        Some(Key::Ok) => {
                            if self.ok_down.is_none() {
                                self.ok_down = Some(Instant::now());
                                self.ok_long = false;
                            }
                        }
                        Some(key) => self.send_key(key),
                        None => {
                            if !event.key_repeat() {
                                log::info!("key scancode {scancode} not used");
                            }
                        }
                    }
                }
                sdl::KEYUP => {
                    if map_key(event.key_scancode()) == Some(Key::Ok)
                        && self.ok_down.take().is_some()
                        && !self.ok_long
                    {
                        self.send_key(Key::Ok);
                    }
                }
                _ => {}
            }
        }

        /// Another app is in front: stop drawing, keep the glyphs and icons for the way back.
        fn set_hidden(&mut self) {
            if self.visible {
                self.visible = false;
                self.shell.sleep(true);
                log::info!("hidden: drawing paused");
            }
        }

        fn set_visible(&mut self) {
            if !self.visible {
                self.visible = true;
                self.shell.wake(self.size.0, self.size.1);
                self.refresh_system_settings();
                self.sync_catalog();
                log::info!("visible again");
            }
        }

        fn refresh_system_settings(&mut self) {
            self.shell.set_locales(&self.bridge.locales());
            self.shell.set_clock(self.bridge.clock());
            if let Some(radios) = self.bridge.radios() {
                self.shell.set_radios(radios.0, radios.1);
            }
            self.refresh_levels();
        }

        /// Scans the apps again and applies the difference (webOS has no package-change events
        /// HOME listens to; the scan is one Luna call).
        fn sync_catalog(&mut self) {
            let discovered = self.bridge.scan_all();
            if !discovered.is_empty() {
                self.shell.rescan(&discovered, unix_seconds());
            }
        }

        fn refresh_levels(&mut self) {
            self.levels_read = Instant::now();
            let volume = self.bridge.volume();
            let backlight = self.bridge.backlight();
            if let (Some(volume), Some(backlight)) = (volume, backlight) {
                self.shell.set_levels(backlight, volume, None);
            }
        }

        fn send_key(&mut self, key: Key) {
            let now = self.now_ms();
            for effect in self.shell.key(key, now) {
                self.run_effect(effect);
            }
            let controls = matches!(
                self.shell.app().screen(),
                lume_core::app::Screen::ControlCenter
            );
            if controls && !self.in_controls {
                self.refresh_levels();
                if let Some(radios) = self.bridge.radios() {
                    self.shell.set_radios(radios.0, radios.1);
                }
            }
            self.in_controls = controls;
        }

        fn run_effect(&mut self, effect: Effect) {
            match effect {
                Effect::OpenApp { package, .. } => {
                    if !self.bridge.launch(&package) {
                        let now = self.now_ms();
                        self.shell.app_not_found(now);
                        self.sync_catalog();
                    }
                }
                Effect::OpenAndroidSettings => {
                    if !self.bridge.launch("com.palm.app.settings") {
                        log::warn!("the TV settings could not be opened");
                    }
                }
                Effect::OpenTvSetting { .. } => {
                    // webOS has no deep links into settings panes HOME knows; the app opens whole.
                    if !self.bridge.launch("com.palm.app.settings") {
                        log::warn!("the TV settings could not be opened");
                    }
                }
                Effect::OpenInputs => {
                    // LG's Home Dashboard lists the inputs.
                    if !self.bridge.launch("com.webos.app.homeconnect") {
                        log::warn!("the inputs screen could not be opened");
                    }
                }
                Effect::OpenPanel(name) => {
                    let id = match name {
                        "airplay" => "airplay",
                        "mirror" => "com.webos.app.screensharing",
                        other => {
                            log::warn!("no webOS screen for panel {other:?}");
                            return;
                        }
                    };
                    if !self.bridge.launch(id) {
                        log::warn!("the {name} screen could not be opened");
                    }
                }
                Effect::OpenAppInfo { package } => {
                    log::info!("app info of {package}: not available on webOS");
                }
                Effect::Uninstall { package } => {
                    // Deliberately inert: a silent appInstallService/remove would delete without
                    // the confirmation Android's uninstaller gives.
                    log::info!("uninstall of {package}: not wired on webOS");
                }
                Effect::SetVolume(percent) => {
                    if !self.bridge.set_volume(percent) {
                        self.refresh_levels();
                    }
                }
                Effect::SetBrightness(percent) => {
                    if !self.bridge.set_backlight(percent) {
                        self.refresh_levels();
                    }
                }
                Effect::SetAutoBrightness(_) => {
                    // No light-sensor API was found on this TV; the toggle stays inert.
                    self.refresh_levels();
                }
                Effect::Sleep => {
                    if !self.bridge.screen_off() {
                        log::warn!("the screen could not be turned off");
                    }
                }
                Effect::SleepTimer(minutes) => {
                    self.sleep_at = (minutes > 0)
                        .then(|| Instant::now() + Duration::from_secs(u64::from(minutes) * 60));
                }
                Effect::SaveConfig | Effect::SaveUsage | Effect::SaveKeys => {}
            }
        }

        fn prepare_wallpaper(&mut self) {
            if self.shell.app().in_splash() && !self.backend.has_wallpaper() {
                let id = self.shell.app().config.wallpaper;
                self.backend.poll_wallpaper(id);
            }
        }

        fn load_icons_batch(&mut self) {
            let now = self.now_ms();
            let budget = if self.shell.app().is_animating(now) {
                FADE_BATCH
            } else if self.shell.app().in_splash() {
                SPLASH_BATCH
            } else {
                IDLE_BATCH
            };
            let start = Instant::now();
            loop {
                self.load_icons(start, budget);
                let time_left = self.shell.splash_time_left(self.now_ms());
                if !self.icons_pending
                    || start.elapsed() >= budget
                    || time_left.is_some_and(|left| left < SPLASH_MARGIN_MS)
                {
                    break;
                }
            }
        }

        fn load_icons(&mut self, started: Instant, budget: Duration) {
            let now = self.now_ms();
            let layout = self.shell.icon_layout();
            let dock = self
                .shell
                .dock_icons_wanted(&self.dock_failed, ICONS_PER_TURN);
            for (package, _) in &dock {
                if started.elapsed() >= budget {
                    break;
                }
                let pixels = self.bridge.dock_icon(package, layout.dock_cell);
                let region = pixels.and_then(|pixels| {
                    self.shell
                        .reserve_dock_icon(package, now)
                        .map(|r| (r, pixels))
                });
                match region {
                    Some((region, pixels)) => {
                        self.backend.ensure_image(layout.width, layout.height);
                        self.backend.update_image(region, &pixels);
                    }
                    None => {
                        self.dock_failed.insert(package.clone());
                        self.shell.icon_failed(package, true);
                    }
                }
            }
            if !dock.is_empty() {
                self.icons_pending = true;
                return;
            }
            let wanted = self.shell.icons_wanted(&self.icons_failed, ICONS_PER_TURN);
            for (package, _) in &wanted {
                if started.elapsed() >= budget {
                    break;
                }
                let art = self
                    .bridge
                    .tile_art(package, layout.cell)
                    .filter(|(p, _)| p.len() == (layout.cell[0] * layout.cell[1] * 4) as usize);
                let region = art
                    .as_ref()
                    .and_then(|(_, art)| self.shell.reserve_icon(package, *art, now));
                match (art, region) {
                    (Some((pixels, _)), Some(region)) => {
                        self.backend.ensure_image(layout.width, layout.height);
                        self.backend.update_image(region, &pixels);
                    }
                    _ => {
                        self.icons_failed.insert(package.clone());
                        self.shell.icon_failed(package, false);
                    }
                }
            }
            self.icons_pending = !self.shell.icons_wanted(&self.icons_failed, 1).is_empty()
                || !self
                    .shell
                    .dock_icons_wanted(&self.dock_failed, 1)
                    .is_empty();
        }

        fn draw(&mut self) {
            let now = self.now_ms();
            let Some(frame) = self.shell.build_frame(now) else {
                return;
            };
            let Some(viewport) = self.shell.viewport() else {
                return;
            };
            if let Some(text) = self.shell.text_engine() {
                self.backend.sync_text_atlas(text);
            }
            let background = match lume_core::theme::THEME.app.base.background {
                Some(lume_core::theme::Paint::Solid(color)) => color,
                _ => vars::BG,
            };
            self.backend.prepare(&frame);
            let _ = self.backend.take_content_changed();
            // Full redraw: no EGL buffer-age through SDL yet. Measured before optimizing.
            self.backend
                .render_damaged(&viewport, background, &frame, None);
            self.window.swap();
            if let Some(from) = self.first_frame_from.take() {
                log::info!("first frame {} ms after start", from.elapsed().as_millis());
            }
        }

        fn device_info(&mut self) -> lume_core::device_info::DeviceInfo {
            use lume_core::device_info::{DeviceInfo, Field};
            let mut info = DeviceInfo::default();
            info.set_text(Field::Brand, "LG");
            if let Some((model, firmware, webos)) = self.bridge.system_info() {
                info.set_text(Field::Model, &model);
                info.set_text(Field::Firmware, &format!("{firmware} (webOS {webos})"));
            }
            let (gpu, gl) = self.backend.gpu_strings();
            info.set_text(Field::Gpu, &gpu);
            info.set_text(Field::OpenGl, &gl);
            info.set_text(
                Field::Resolution,
                &format!("{} × {}", self.size.0, self.size.1),
            );
            info
        }
    }

    fn unix_seconds() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}
