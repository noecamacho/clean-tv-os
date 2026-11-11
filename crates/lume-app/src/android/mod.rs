//! The Android event loop.
//!
//! The loop blocks in `poll_events` until a key arrives, the window or the activity changes,
//! or the next deadline the launcher reported (an animation, the clock's minute).
//! A frame is drawn only when something changed, and everything that holds GPU memory is
//! released when the window goes away.

mod bridge;
mod device_info;
mod egl;

use android_activity::input::{InputEvent, KeyAction};
use android_activity::{AndroidApp, AndroidAppWaker, InputStatus, MainEvent, PollEvent};
use bridge::{unix_seconds, Bridge};
use egl::Graphics;
use lume_core::app::{Effect, Key};
use lume_core::render::{Frame, Gles2Backend, Rect, FULL_DAMAGE};
use lume_core::shell::protocol::Changed;
use lume_core::shell::Shell;
use lume_core::theme::{vars, Color};
use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Set by Java when the Home button is pressed while lume is already open.
static HOME_PRESSED: AtomicBool = AtomicBool::new(false);
static WAKER: Mutex<Option<AndroidAppWaker>> = Mutex::new(None);
/// Set by the wallpaper's worker thread when a wallpaper is ready: the loop draws a frame to swap it in.
static WALLPAPER_READY: AtomicBool = AtomicBool::new(false);

fn wallpaper_ready() {
    WALLPAPER_READY.store(true, Ordering::SeqCst);
    if let Ok(guard) = WAKER.lock() {
        if let Some(waker) = guard.as_ref() {
            waker.wake();
        }
    }
}

/// The app, for reading the wallpapers from the APK's assets.
static APP: OnceLock<AndroidApp> = OnceLock::new();

/// Reads `assets/wallpaper/<name>` from the APK (the NDK's `AAssetManager`; no Java).
fn read_wallpaper_asset(name: &str) -> Option<Vec<u8>> {
    let path = std::ffi::CString::new(format!("wallpaper/{name}")).ok()?;
    let mut asset = APP.get()?.asset_manager().open(&path)?;
    let mut bytes = Vec::with_capacity(asset.length());
    std::io::Read::read_to_end(&mut asset, &mut bytes).ok()?;
    Some(bytes)
}

/// Called by `LumeActivity.onNewIntent` (Java thread) to say Home was pressed.
#[no_mangle]
pub extern "system" fn Java_app_lume_tv_LumeActivity_nativeHomePressed(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    HOME_PRESSED.store(true, Ordering::SeqCst);
    if let Ok(guard) = WAKER.lock() {
        if let Some(waker) = guard.as_ref() {
            waker.wake();
        }
    }
}

/// Icons fetched per call, so keys are still answered while they load.
const ICONS_PER_TURN: usize = 8;
/// How long a turn of the loop may load icons: while the splash waits for them (nothing moves and
/// nobody sees them come), with the interface at rest (one frame, so a key is not kept waiting),
/// and while a faint fade runs (the splash logo or an icon coming in: a slice of a frame).
const SPLASH_BATCH: Duration = Duration::from_millis(40);
const IDLE_BATCH: Duration = Duration::from_millis(16);
const FADE_BATCH: Duration = Duration::from_millis(6);
/// No icon is started this close to the splash's deadline (one can take this long).
const SPLASH_MARGIN_MS: u64 = 60;

/// `mallopt` option that returns freed heap pages to the system (bionic, Android 10+).
const M_PURGE: libc::c_int = -101;

extern "C" {
    /// Bionic's allocator tuning call.
    fn mallopt(option: libc::c_int, value: libc::c_int) -> libc::c_int;
}

/// Gives freed memory back to the system, so the process stays small while in the background.
fn purge_memory() {
    // SAFETY: mallopt only adjusts allocator behavior; unsupported options are ignored.
    unsafe {
        mallopt(M_PURGE, 0);
    }
}

/// Where the time of loading the icons of a window goes: fetching them from the platform (the
/// bridge) and uploading them to the GPU.
#[derive(Default)]
struct IconTimes {
    /// When the window appeared, until the log line is written.
    from: Option<Instant>,
    count: u32,
    bridge: Duration,
    upload: Duration,
    /// The slowest single fetch.
    slowest: Duration,
}

impl IconTimes {
    fn fetched(&mut self, took: Duration) {
        self.count += 1;
        self.bridge += took;
        self.slowest = self.slowest.max(took);
    }
}

/// The state of the app while its event loop runs.
struct Runtime {
    app: AndroidApp,
    bridge: Bridge,
    shell: Shell,
    graphics: Option<Graphics>,
    /// The last frame drawn and what each of the last frames changed (newest first), to redraw
    /// only that: see `draw`.
    last_frame: Option<Frame>,
    damages: std::collections::VecDeque<Option<Rect>>,
    /// Apps whose icon could not be loaded in this window, so they are not asked again.
    icons_failed: HashSet<String>,
    /// Apps whose dock icon the platform could not give.
    dock_failed: HashSet<String>,
    /// Icons are still missing: the loop does not wait for events.
    icons_pending: bool,
    started: Instant,
    /// When the current window appeared, until its first frame is drawn and logged.
    first_frame_from: Option<Instant>,
    /// How the icons of the current window were loaded, logged once they all are.
    icon_times: IconTimes,
    /// When the frames of the reveal after the splash were shown, to log how many were late.
    reveal_frames: Vec<Instant>,
    /// When the current window appeared, until the interface is ready (the splash ended) and logged.
    ready_from: Option<Instant>,
    /// Size of the render buffer, once a window exists.
    running: bool,
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("lume"),
    );
    std::panic::set_hook(Box::new(|info| log::error!("panic: {info}")));
    log::info!("lume {} starting", lume_version::VERSION);
    *WAKER.lock().expect("waker lock") = Some(app.create_waker());
    let _ = APP.set(app.clone());
    lume_core::wallpaper::set_asset_reader(read_wallpaper_asset);
    lume_core::wallpaper::set_ready_notifier(wallpaper_ready);

    let Some(files) = app.internal_data_path() else {
        log::error!("no internal data directory");
        return;
    };
    // The app's cache directory (next to `files/`): the decoded wallpaper is kept there between wake-ups.
    if let Some(cache) = files.parent().map(|parent| parent.join("cache")) {
        lume_core::wallpaper::set_cache_dir(cache);
    }
    let bridge = Bridge::new(&app);
    let package = bridge.package_name();
    let mut shell = Shell::start(
        &files,
        &package,
        &bridge.locales(),
        bridge.clock(),
        &mut || bridge.scan_all(None),
    );

    shell.set_inputs_available(bridge.inputs_available());

    let mut runtime = Runtime {
        app: app.clone(),
        bridge,
        shell,
        graphics: None,
        last_frame: None,
        damages: std::collections::VecDeque::new(),
        icons_failed: HashSet::new(),
        dock_failed: HashSet::new(),
        icons_pending: false,
        started: Instant::now(),
        first_frame_from: None,
        ready_from: None,
        reveal_frames: Vec::new(),
        icon_times: IconTimes::default(),
        running: true,
    };
    runtime.run();
    log::info!("lume stopped");
}

impl Runtime {
    fn now_ms(&self) -> u64 {
        self.started.elapsed().as_millis() as u64
    }

    fn run(&mut self) {
        while self.running {
            // A wallpaper going up to the GPU slice by slice needs a frame for each slice: asking here, before
            // the timeout is computed, keeps the loop from going to sleep between them.
            if self
                .graphics
                .as_mut()
                .is_some_and(|g| g.backend().wallpaper_uploading())
            {
                self.shell.invalidate();
            }
            let busy = self.shell.is_busy(self.now_ms());
            let timeout = if self.icons_pending && !busy {
                Some(Duration::ZERO)
            } else {
                // The statistics are read only while the settings are on screen.
                [
                    self.shell.next_timeout(self.now_ms()),
                    self.shell.stats_timeout(self.now_ms()),
                ]
                .into_iter()
                .flatten()
                .min()
            };
            let app = self.app.clone();
            app.poll_events(timeout, |event| self.on_event(event));
            self.process_input();
            if WALLPAPER_READY.swap(false, Ordering::SeqCst) {
                self.shell.invalidate();
            }
            if HOME_PRESSED.swap(false, Ordering::SeqCst) {
                self.send_key(Key::Home);
            }
            // The clock's seconds move on by themselves; the shell only asks for a frame when the minute turns.
            self.shell.tick_clock(unix_seconds());
            if self.graphics.is_some() && self.shell.frame_due(self.now_ms()) {
                self.draw();
            }
            let now = self.now_ms();
            if self.shell.stats_due(now) {
                if let Some(stats) = self.bridge.stats() {
                    self.shell.set_stats(stats, now);
                } else {
                    // Try again in a while instead of on every turn of the loop.
                    self.shell.stats_failed(now);
                }
            }
            if self.shell.device_info_due(now) {
                let gpu = self.graphics.as_mut().map(|g| g.backend().gpu_strings());
                let info = device_info::read(gpu, self.bridge.display());
                self.shell.set_device_info(info);
            }
            // Loading an icon takes tens of milliseconds: it waits for the animation to end, or the
            // frames would be late and the movement would stutter.
            if !self.shell.is_busy(self.now_ms()) {
                self.prepare_wallpaper();
                self.load_icons_batch();
            }
        }
    }

    fn on_event(&mut self, event: PollEvent<'_>) {
        let PollEvent::Main(event) = event else {
            return;
        };
        match event {
            MainEvent::InitWindow { .. } => self.create_window(),
            MainEvent::TerminateWindow { .. } => self.destroy_window(),
            MainEvent::WindowResized { .. } | MainEvent::RedrawNeeded { .. } => {
                if self.graphics.is_some() {
                    self.create_window_size();
                }
            }
            MainEvent::Resume { .. } => self.on_resume(),
            MainEvent::ConfigChanged { .. } => self.refresh_system_settings(),
            // Anything that keeps GPU memory goes away as soon as lume is not on screen.
            MainEvent::Stop => self.destroy_window(),
            MainEvent::LowMemory => {
                if self.graphics.is_none() {
                    purge_memory();
                }
            }
            MainEvent::Destroy => self.running = false,
            _ => {}
        }
    }

    /// Creates the EGL context and the renderer for the new window.
    fn create_window(&mut self) {
        let Some(window) = self.app.native_window() else {
            return;
        };
        // Render at most 1920x1080; the system scales the buffer up on bigger screens.
        let (width, height) =
            Shell::render_size(window.width().max(1) as u32, window.height().max(1) as u32);
        if let Err(e) = window.set_buffers_geometry(
            width as i32,
            height as i32,
            Some(ndk::hardware_buffer_format::HardwareBufferFormat::R8G8B8A8_UNORM),
        ) {
            log::warn!("cannot set the buffer size: {e}");
        }
        match Graphics::new(window.ptr().as_ptr().cast()) {
            Ok(graphics) => {
                self.graphics = Some(graphics);
                if let Some(graphics) = self.graphics.as_mut() {
                    graphics.backend().set_render_size(width, height);
                    graphics.backend().set_async_wallpaper(true);
                }
                self.first_frame_from = Some(Instant::now());
                self.ready_from = self.first_frame_from;
                self.icon_times = IconTimes {
                    from: self.first_frame_from,
                    ..IconTimes::default()
                };
                self.shell.wake(width, height);
                log::info!("window {width}x{height}");
            }
            Err(e) => log::error!("cannot start the renderer: {e}"),
        }
    }

    /// The window changed size: the buffer size and the scale are recalculated.
    fn create_window_size(&mut self) {
        let Some(window) = self.app.native_window() else {
            return;
        };
        let (width, height) =
            Shell::render_size(window.width().max(1) as u32, window.height().max(1) as u32);
        let _ = window.set_buffers_geometry(
            width as i32,
            height as i32,
            Some(ndk::hardware_buffer_format::HardwareBufferFormat::R8G8B8A8_UNORM),
        );
        if let Some(graphics) = self.graphics.as_mut() {
            graphics.backend().set_render_size(width, height);
        }
        self.shell.wake(width, height);
    }

    /// Releases the renderer, the glyph atlas and the heap pages they used.
    fn destroy_window(&mut self) {
        let had_graphics = self.graphics.take().is_some();
        self.icons_failed.clear();
        self.dock_failed.clear();
        self.icons_pending = false;
        self.shell.sleep();
        if had_graphics {
            purge_memory();
            log::info!("window gone: renderer, glyph atlas and fonts released (dormant)");
        }
    }

    fn on_resume(&mut self) {
        self.refresh_system_settings();
        self.sync_catalog();
    }

    /// Language, clock format and time zone may have changed while lume was away.
    fn refresh_system_settings(&mut self) {
        self.shell.set_locales(&self.bridge.locales());
        self.shell.set_clock(self.bridge.clock());
    }

    /// Picks up apps installed, updated or removed since the last time.
    fn sync_catalog(&mut self) {
        let now = unix_seconds();
        match self.bridge.changed() {
            Changed::Unknown => {
                let discovered = self.bridge.scan_all(Some(&self.shell.app().catalog));
                self.shell.rescan(&discovered, now);
            }
            Changed::Packages(packages) if packages.is_empty() => {}
            Changed::Packages(packages) => {
                let (mut changed, mut removed) = (Vec::new(), Vec::new());
                for package in packages {
                    let entries = self.bridge.query(&package);
                    if entries.is_empty() {
                        removed.push(package);
                    } else {
                        changed.extend(entries);
                    }
                }
                self.shell.apply_changes(&changed, &removed, now);
            }
        }
    }

    fn process_input(&mut self) {
        let mut pressed_codes = Vec::new();
        match self.app.input_events_iter() {
            Ok(mut iter) => loop {
                let more = iter.next(|event| match event {
                    InputEvent::KeyEvent(key) => {
                        let code: u32 = key.key_code().into();
                        let code = code as i32;
                        let app = self.shell.app();
                        // The keys lume does not use (volume, power...) stay for the system, and
                        // so they do not disappear; while a key is being recorded, all are lume's.
                        let used = app.is_listening_key()
                            || app.keys.action_for(code).is_some()
                            || lume_core::keys::from_android(code).is_some();
                        if !used {
                            // Which code a remote sends for a button lume does not use, for
                            // mapping remotes that differ (see docs/device-notes.md).
                            if key.action() == KeyAction::Down {
                                log::info!("key {code} not used");
                            }
                            return InputStatus::Unhandled;
                        }
                        if key.action() == KeyAction::Down {
                            pressed_codes.push(code);
                        }
                        InputStatus::Handled
                    }
                    _ => InputStatus::Unhandled,
                });
                if !more {
                    break;
                }
            },
            Err(e) => log::warn!("no input events: {e:?}"),
        }
        for code in pressed_codes {
            self.send_keycode(code);
        }
    }

    fn send_keycode(&mut self, code: i32) {
        let now = self.now_ms();
        for effect in self.shell.raw_key(code, now) {
            self.run_effect(effect);
        }
    }

    fn send_key(&mut self, key: Key) {
        let now = self.now_ms();
        for effect in self.shell.key(key, now) {
            self.run_effect(effect);
        }
    }

    fn run_effect(&mut self, effect: Effect) {
        match effect {
            Effect::OpenApp { package, activity } => {
                if !self.bridge.start_app(&package, &activity) {
                    // It was uninstalled meanwhile: say so, and refresh the catalog.
                    let now = self.now_ms();
                    self.shell.app_not_found(now);
                    let discovered = self.bridge.scan_all(Some(&self.shell.app().catalog));
                    self.shell.rescan(&discovered, unix_seconds());
                }
            }
            Effect::OpenAndroidSettings => {
                if !self.bridge.open_android_settings() {
                    log::warn!("the Android settings could not be opened");
                }
            }
            Effect::OpenTvSetting { setting } => {
                if !self.bridge.open_setting(setting.as_str()) {
                    log::warn!("the TV setting {setting:?} could not be opened");
                }
            }
            Effect::OpenInputs => {
                if !self.bridge.open_inputs() {
                    log::warn!("the TV inputs panel could not be opened");
                }
            }
            Effect::OpenAppInfo { package } => {
                if !self.bridge.open_app_info(&package) {
                    log::warn!("the app info of {package} could not be opened");
                }
            }
            Effect::Uninstall { package } => {
                if !self.bridge.uninstall(&package) {
                    log::warn!("the uninstaller of {package} could not be opened");
                }
            }
            // The shell already saved these.
            Effect::SaveConfig | Effect::SaveUsage | Effect::SaveKeys => {}
        }
    }

    /// While the splash is on screen (it does not draw the wallpaper), uploads the wallpaper texture
    /// for the reveal: uploaded by the reveal's first frame, it made that frame ~70 ms late.
    fn prepare_wallpaper(&mut self) {
        if let Some(graphics) = self.graphics.as_mut() {
            if self.shell.app().in_splash() && !graphics.backend().has_wallpaper() {
                graphics
                    .backend()
                    .poll_wallpaper(self.shell.app().config.wallpaper);
            }
        }
    }

    /// Loads icons for a turn of the loop, as many as fit in the turn's budget (see
    /// [`SPLASH_BATCH`]); the screen is drawn once for all of them. Loading one icon per turn cost a
    /// frame per icon, which is what made them appear one by one. During the splash it stops in time
    /// for the deadline, which an icon on its way would delay.
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

    /// Fetches the next few missing icons and uploads them to the icon texture.
    fn load_icons(&mut self, started: Instant, budget: Duration) {
        let now = self.now_ms();
        let Some(graphics) = self.graphics.as_mut() else {
            self.icons_pending = false;
            return;
        };
        // The dock is what Home shows: its icons come first, a couple per turn, so the first frames
        // are drawn right away with the app color and initial.
        let layout = self.shell.icon_layout();
        let dock = self
            .shell
            .dock_icons_wanted(&self.dock_failed, ICONS_PER_TURN);
        for (package, activity) in &dock {
            if started.elapsed() >= budget {
                break;
            }
            let fetch = Instant::now();
            let pixels = self.bridge.dock_icon(package, activity, layout.dock_cell);
            self.icon_times.fetched(fetch.elapsed());
            let region = pixels.and_then(|pixels| {
                self.shell
                    .reserve_dock_icon(package, now)
                    .map(|r| (r, pixels))
            });
            match region {
                Some((region, pixels)) => {
                    let upload = Instant::now();
                    let backend = graphics.backend();
                    backend.ensure_image(layout.width, layout.height);
                    backend.update_image(region, &pixels);
                    self.icon_times.upload += upload.elapsed();
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
        for (package, activity) in &wanted {
            if started.elapsed() >= budget {
                break;
            }
            let fetch = Instant::now();
            let art = self
                .bridge
                .tile_art(package, activity, layout.cell)
                .filter(|(p, _)| p.len() == (layout.cell[0] * layout.cell[1] * 4) as usize);
            self.icon_times.fetched(fetch.elapsed());
            let region = art
                .as_ref()
                .and_then(|(_, art)| self.shell.reserve_icon(package, *art, now));
            match (art, region) {
                (Some((pixels, _)), Some(region)) => {
                    let upload = Instant::now();
                    let backend = graphics.backend();
                    backend.ensure_image(layout.width, layout.height);
                    backend.update_image(region, &pixels);
                    self.icon_times.upload += upload.elapsed();
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
        if !self.icons_pending {
            let times = &mut self.icon_times;
            if let Some(from) = times.from.take() {
                log::info!(
                    "icons: {} in {} ms after the window (bridge {} ms, slowest {} ms; upload {} ms)",
                    times.count,
                    from.elapsed().as_millis(),
                    times.bridge.as_millis(),
                    times.slowest.as_millis(),
                    times.upload.as_millis()
                );
            }
        }
    }

    fn draw(&mut self) {
        let now = self.now_ms();
        // The frame that ends the reveal is drawn after it stops being "revealing": it counts too.
        let revealing = self.shell.is_revealing();
        let Some(frame) = self.shell.build_frame(now) else {
            return;
        };
        let (Some(graphics), Some(viewport)) = (self.graphics.as_mut(), self.shell.viewport())
        else {
            return;
        };
        if let Some(text) = self.shell.text_engine() {
            graphics.backend().sync_text_atlas(text);
        }
        let background = match lume_core::theme::THEME.app.base.background {
            Some(lume_core::theme::Paint::Solid(color)) => color,
            _ => vars::BG,
        };
        let _: Color = background;
        // Only what changed since the buffer's own content is redrawn: the fill rate of the
        // TV's GPU does not reach a whole 1080p frame of glass every 16 ms.
        graphics.backend().prepare(&frame);
        let content_changed = graphics.backend().take_content_changed();
        let damage = match &self.last_frame {
            Some(last) if !content_changed => frame.damage(last),
            _ => Some(FULL_DAMAGE),
        };
        let age = graphics.egl.buffer_age();
        let repaint = if age == 0 || age > self.damages.len() + 1 {
            Some(FULL_DAMAGE)
        } else {
            self.damages
                .iter()
                .take(age - 1)
                .fold(damage, |all, d| union(all, *d))
        };
        self.damages.push_front(damage);
        self.damages.truncate(4);
        self.last_frame = Some(frame);
        let frame = self.last_frame.as_ref().expect("just set");
        // Nothing changed: one pixel still goes through the swap, which is what paces the loop
        // to the screen while an animation runs.
        let mut scissor = repaint.map_or([0; 4], |r| Gles2Backend::device_rect(&r, &viewport));
        if scissor[2] == 0 || scissor[3] == 0 {
            scissor = [0, 0, 1, 1];
        }
        let whole = scissor[2] as u32 >= viewport.width && scissor[3] as u32 >= viewport.height;
        if !whole {
            graphics.egl.set_damage(scissor);
        }
        graphics.backend().render_damaged(
            &viewport,
            background,
            frame,
            (!whole).then_some(scissor),
        );
        let swapped = graphics.egl.swap_buffers();
        if let Err(e) = &swapped {
            log::warn!("{e}");
        } else if let Some(from) = self.first_frame_from.take() {
            log::info!(
                "first frame: {} ms after the window, {} ms after the process started",
                from.elapsed().as_millis(),
                self.started.elapsed().as_millis()
            );
        }
        if swapped.is_ok() && (revealing || self.shell.is_revealing()) {
            self.reveal_frames.push(Instant::now());
        }
        if self.shell.take_interface_ready() {
            let frames = std::mem::take(&mut self.reveal_frames);
            let gaps: Vec<u128> = frames
                .windows(2)
                .map(|pair| (pair[1] - pair[0]).as_millis())
                .collect();
            let late = gaps.iter().filter(|gap| **gap >= 33).count();
            if let Some(from) = self.ready_from.take() {
                log::info!(
                    "interface ready: {} ms after the window, {} ms after the process started; reveal: {} frames, {} of 33 ms or more {:?}",
                    from.elapsed().as_millis(),
                    self.started.elapsed().as_millis(),
                    frames.len(),
                    late,
                    gaps
                );
            }
        }
    }
}

/// The bounds of two damage rects (`None`: nothing changed).
fn union(a: Option<Rect>, b: Option<Rect>) -> Option<Rect> {
    match (a, b) {
        (Some(a), Some(b)) => {
            let (x, y) = (a.x.min(b.x), a.y.min(b.y));
            Some(Rect::new(
                x,
                y,
                a.right().max(b.right()) - x,
                a.bottom().max(b.bottom()) - y,
            ))
        }
        (a, b) => a.or(b),
    }
}
